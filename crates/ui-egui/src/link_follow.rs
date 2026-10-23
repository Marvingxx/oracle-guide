//! Placed graphics follow their files while the document is open, as in Oracle Edit and Oracle
//! Vector: a logo saved again in Oracle Vector, a photo retouched in Oracle Edit, shows here
//! without Update Link.
//!
//! Every two seconds the active document's linked files are stamped (size and modification time:
//! nothing is read). When a stamp moved since the last look, the Links rows are asked for their
//! status and each modified graphic is updated, one undo step each. A file seen for the first time
//! is only remembered. Linked text stories are left to Update Link, which asks before replacing
//! edits made to them in the layout.

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::DesignApp;

/// How often linked files are looked at, in seconds.
const EVERY: f64 = 2.0;

/// A file's size and modification time.
type Stamp = (u64, u128);

#[derive(Default)]
pub struct LinkFollow {
    last: f64,
    /// Per document id, the stamps of its linked files when last looked at.
    seen: HashMap<u64, HashMap<PathBuf, Stamp>>,
}

fn stamp(path: &std::path::Path) -> Option<Stamp> {
    let meta = oracle_guide_engine::io::metadata(path).ok()?;
    let time = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos();
    Some((meta.len(), time))
}

/// Looks at the active document's linked files at most every [`EVERY`] seconds and updates the
/// graphics whose files changed.
pub fn tick(app: &mut DesignApp, now: f64) {
    if now - app.link_follow.last < EVERY {
        return;
    }
    app.link_follow.last = now;
    let Some(st) = app.session.active() else { return };
    let doc_id = st.uid;
    let document = st.path.clone().map(PathBuf::from);
    let files: HashMap<PathBuf, Stamp> = st
        .doc
        .assets
        .values()
        .filter_map(|a| a.link.as_deref())
        .filter_map(|link| oracle_guide_engine::io::document_path(link, document.as_deref()).ok())
        .filter_map(|path| stamp(&path).map(|s| (path, s)))
        .collect();
    let before = app.link_follow.seen.insert(doc_id, files.clone());
    let moved = before.is_some_and(|before| files.iter().any(|(path, s)| before.get(path).is_some_and(|b| b != s)));
    if !moved {
        return;
    }
    let rows = app.session.execute("links.list", &json!({})).ok().and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let modified: Vec<(u64, String)> = rows
        .iter()
        .filter(|r| r["status"] == "modified")
        .filter_map(|r| Some((r.get("asset").and_then(Value::as_u64)?, r["name"].as_str().unwrap_or_default().to_string())))
        .collect();
    for (asset, name) in modified {
        match app.run("links.update", json!({"asset": asset})) {
            Ok(_) => app.status(crate::i18n::tr(&app.ui.language, "Link updated: {name}").replace("{name}", &name)),
            Err(e) => app.status(e),
        }
    }
}
