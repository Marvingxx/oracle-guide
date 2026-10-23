//! Send to: the Brand Guide hands its document to Oracle Edit (File › Send to Oracle Edit), and
//! takes what the other Oracle apps send it: an Oracle Vector logo placed, linked, in the open guide,
//! following its saves (link_follow). The exchange goes through `oracle_suite_link::inbox`: a small
//! request file the receiving app takes about every second, starting it when it isn't running.

use std::path::PathBuf;

use oracle_suite_link::App;
use oracle_suite_link::inbox::{self, Action};
use serde_json::{Value, json};

use crate::DesignApp;

/// How often the inbox is looked at, in seconds.
const EVERY: f64 = 1.0;

/// The file that stands for the active document: itself when saved as a Brand Guide document and
/// unchanged since, else a copy stored in the library ("Mes créations").
fn document_file(app: &mut DesignApp) -> Result<PathBuf, String> {
    let st = app.session.active().ok_or("no document is open")?;
    if let Some(path) = st.path.as_deref().filter(|p| p.to_ascii_lowercase().ends_with(".oracle-guide") && !st.is_dirty()) {
        return Ok(PathBuf::from(path));
    }
    crate::library_send::store_active(app)
}

/// File › Send to `target`: the guide placed, linked, in `target`'s open document.
pub fn send_active(app: &mut DesignApp, target: App) -> Result<Value, String> {
    let file = document_file(app)?;
    inbox::send(target, Action::Place, &file)?;
    app.status(crate::i18n::tr(&app.ui.language, "Sent to {app}").replace("{app}", target.name()));
    Ok(json!({"sentTo": target.id(), "path": file}))
}

/// Takes what the other Oracle apps sent: each file placed, linked, in the active document (opened
/// as a document when none is open, or when asked to open it). Looks at most every second, on frames
/// the app draws anyway (it never wakes itself for it).
pub fn poll(app: &mut DesignApp, ctx: &egui::Context, now: f64) {
    if now - app.inbox_last < EVERY {
        return;
    }
    app.inbox_last = now;
    let requests = inbox::take(App::Guide);
    if requests.is_empty() {
        return;
    }
    for r in requests {
        let path = r.file.to_string_lossy().into_owned();
        let done = match r.action {
            Action::Place if app.session.active().is_some() => app.run("file.place", json!({"path": path})),
            _ => app.run("file.open", json!({"path": path})),
        };
        if let Err(e) = done {
            app.status(e);
        }
    }
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}
