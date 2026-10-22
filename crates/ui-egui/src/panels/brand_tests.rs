//! Marque panel: link a Brand Kit project, see what differs, update, place a logo.

use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::DesignApp;

fn harness(app: DesignApp) -> Harness<'static, DesignApp> {
    Harness::builder().with_size(vec2(320.0, 720.0)).build_ui_state(|ui, app: &mut DesignApp| super::brand::show(app, ui), app)
}

/// Writes the panel's picture where `ORACLE_SHOTS` points, to look at.
fn shot(h: &mut Harness<'static, DesignApp>, name: &str) {
    if let Some(dir) = std::env::var_os("ORACLE_SHOTS") {
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(img) = h.render() {
            let _ = img.save(std::path::Path::new(&dir).join(format!("{name}.png")));
        }
    }
}

#[test]
fn link_review_and_update_from_the_panel() {
    let dir = std::env::temp_dir().join(format!("og-brand-panel-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("logo.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80"><rect width="200" height="80" fill="#C82029"/></svg>"##).unwrap();
    let brand = json!({"schemaVersion": 1, "projectId": "p", "brandName": "Marque Exemple", "version": "v1",
        "assets": [{"id": "logo-a", "role": "primaryLogo", "label": "Logo principal", "source": "logo.svg"}],
        "colors": [{"id": "c1", "name": "Rouge principal", "hex": "#C82029"}, {"id": "c2", "name": "Noir", "hex": "#1D1D1B"}],
        "fonts": [{"id": "f1", "family": "Inter", "usage": "Texte courant"}]});
    std::fs::write(dir.join("brand.json"), brand.to_string()).unwrap();

    let mut app = DesignApp::new(oracle_guide_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({})).unwrap();
    let mut h = harness(app);
    h.run_steps(3);
    h.get_by_label("Lier un projet de marque…");
    shot(&mut h, "brand-unlinked");
    h.state_mut().run("brand.link", json!({"path": dir.join("brand.json").display().to_string()})).unwrap();
    h.run_steps(3);
    h.get_by_label_contains("mise(s) à jour disponible(s)");
    h.get_by_label("Nuance « Rouge principal »");
    shot(&mut h, "brand-changes");
    h.get_by_label_contains("Mettre à jour (").click();
    h.run_steps(4);
    h.get_by_label("À jour avec le Brand Kit.");
    assert!(h.state().session.doc().unwrap().doc.swatches.iter().any(|s| s.name == "Rouge principal"));
    h.get_by_label("Placer").click();
    h.run_steps(3);
    let d = &h.state().session.doc().unwrap().doc;
    let labelled = d.spreads.iter().flat_map(|s| s.items.iter()).filter(|i| i.label == "oracle:logo:logo-a@primaryLogo").count();
    assert_eq!(labelled, 1, "the logo is placed, linked and labelled");
    shot(&mut h, "brand-in-sync");
    let _ = std::fs::remove_dir_all(&dir);
}
