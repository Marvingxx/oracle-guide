//! Oracle › Marque: the Brand Kit project this guide follows. It shows what differs from the
//! brand and applies only what is ticked (`brand.sync`), inserts the brand's text variables at the
//! caret, and places linked logos, mockup renders and the palette and type blocks. Everything goes
//! through `brand.*` commands; nothing changes on its own.

use oracle_guide_doc::SpreadRef;
use serde_json::{Value, json};

use crate::DesignApp;
use crate::theme::Tokens;

/// The spread and rectangle a new block takes: `size` (pt) centred in the view.
fn view_rect(app: &DesignApp, size: (f64, f64)) -> Option<(Value, [f64; 4])> {
    let (spread, c) = crate::canvas::view_centre(app)?;
    let spread = match spread {
        SpreadRef::Doc(i) => json!(i),
        SpreadRef::Parent(i) => json!({"kind": "parent", "index": i}),
    };
    Some((spread, [c.x - size.0 / 2.0, c.y - size.1 / 2.0, c.x + size.0 / 2.0, c.y + size.1 / 2.0]))
}

/// The selected frame, when one is selected (a logo goes into it).
fn selected_frame(app: &DesignApp) -> Option<u64> {
    let st = app.session.active()?;
    match st.selection.items.as_slice() {
        [one] => Some(one.0),
        _ => None,
    }
}

fn report(app: &mut DesignApp, r: Result<Value, String>, done: &str) {
    match r {
        Ok(v) => {
            let warnings: Vec<String> = v["warnings"].as_array().map(|w| w.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
            let missing: Vec<String> = v["missingFonts"].as_array().map(|w| w.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
            if !missing.is_empty() {
                app.status(format!("{done} — police(s) absente(s) de cet ordinateur : {} (remplacées à l'affichage)", missing.join(", ")));
            } else if warnings.is_empty() {
                app.status(done);
            } else {
                app.status(format!("{done} — {}", warnings.join(" ; ")));
            }
        }
        Err(e) => app.status(&e),
    }
}

fn place(app: &mut DesignApp, cmd: &str, key: &str, id: &str, size: (f64, f64), done: &str) {
    let mut p = json!({ key: id });
    if let Some(f) = selected_frame(app).filter(|_| cmd != "brand.placePalette" && cmd != "brand.placeTypes") {
        p["frame"] = json!(f);
    } else {
        // Centred in the view; without a view (no canvas yet), at the first page's top left.
        let (spread, rect) = view_rect(app, size).unwrap_or((json!(0), [72.0, 72.0, 72.0 + size.0, 72.0 + size.1]));
        p["spread"] = spread;
        p["rect"] = json!(rect);
    }
    let r = app.run(cmd, p);
    report(app, r, done);
}

pub fn show(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let Some(st) = app.session.active() else {
        ui.label(egui::RichText::new("Aucun document ouvert.").color(t.text_dim));
        return;
    };
    // The status reads files on disk: at most twice a second, or when the document changes.
    let key = egui::Id::new("brand_panel");
    let now = ui.input(|i| i.time);
    let rev = (st.uid, st.revision);
    let cached: Option<(f64, (u64, u64), Value)> = ui.data(|d| d.get_temp(key));
    let status = match cached {
        Some((at, r, v)) if r == rev && now - at < 0.5 => v,
        _ => {
            let v = app.session.execute("brand.status", &json!({})).unwrap_or_else(|e| json!({"linked": true, "error": e.to_string()}));
            ui.data_mut(|d| d.insert_temp(key, (now, rev, v.clone())));
            v
        }
    };

    if let Some(e) = status["error"].as_str() {
        ui.label(egui::RichText::new(format!("Projet de marque illisible : {e}")).color(egui::Color32::from_rgb(220, 80, 70)));
    }
    if status["linked"] != true {
        ui.label(egui::RichText::new("Ce document ne suit aucune marque.").color(t.text_dim));
        ui.label(egui::RichText::new("Liez le projet Oracle Brand Kit (son brand.json) : ses couleurs, typographies, règles, logos et mockups deviennent des blocs liés, mis à jour quand vous le demandez.").size(11.5).color(t.text_dim));
        if ui.button("Lier un projet de marque…").clicked() {
            link(app);
        }
        return;
    }
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(status["brandName"].as_str().unwrap_or_default()).strong().color(t.text_strong));
        if ui.small_button("Changer…").clicked() {
            link(app);
        }
        if ui.small_button("Délier").on_hover_text("Les blocs restent tels quels, sans suivre la marque").clicked() {
            let r = app.run("brand.unlink", json!({}));
            report(app, r, "Document délié de la marque.");
        }
    });
    ui.label(egui::RichText::new(status["path"].as_str().unwrap_or_default()).size(10.5).color(t.text_dim));
    ui.separator();

    // What differs, ticked by default.
    let changes = status["changes"].as_array().cloned().unwrap_or_default();
    let applicable: Vec<&Value> = changes.iter().filter(|c| c["apply"] == true).collect();
    let infos: Vec<&Value> = changes.iter().filter(|c| c["apply"] != true).collect();
    let skip_key = egui::Id::new("brand_panel_skip");
    let mut skipped: Vec<String> = ui.data(|d| d.get_temp(skip_key)).unwrap_or_default();
    if applicable.is_empty() {
        ui.label(egui::RichText::new("À jour avec le Brand Kit.").color(t.text));
    } else {
        ui.label(egui::RichText::new(format!("{} mise(s) à jour disponible(s)", applicable.len())).strong());
        egui::ScrollArea::vertical().id_salt("brand_changes").max_height(220.0).show(ui, |ui| {
            for c in &applicable {
                let k = c["key"].as_str().unwrap_or_default().to_string();
                let mut on = !skipped.contains(&k);
                let text = c["label"].as_str().unwrap_or_default();
                if ui.checkbox(&mut on, text).changed() {
                    if on {
                        skipped.retain(|x| x != &k);
                    } else {
                        skipped.push(k.clone());
                    }
                }
                let (from, to) = (c["from"].as_str().unwrap_or_default(), c["to"].as_str().unwrap_or_default());
                if c["kind"] == "text" || c["kind"] == "color" {
                    ui.label(egui::RichText::new(format!("    {from}  >  {to}")).size(10.5).color(t.text_dim));
                }
            }
        });
        let picked: Vec<String> = applicable.iter().filter_map(|c| c["key"].as_str()).filter(|k| !skipped.iter().any(|s| s == k)).map(str::to_string).collect();
        let b = egui::Button::new(egui::RichText::new(format!("Mettre à jour ({})", picked.len())).color(egui::Color32::WHITE)).fill(t.accent_strong);
        if ui.add_enabled(!picked.is_empty(), b).clicked() {
            let r = app.run("brand.sync", json!({"keys": picked}));
            report(app, r, "Guide mis à jour depuis le Brand Kit.");
            skipped.clear();
        }
    }
    ui.data_mut(|d| d.insert_temp(skip_key, skipped));
    for c in infos {
        ui.label(egui::RichText::new(format!("Attention : {}", c["label"].as_str().unwrap_or_default())).size(11.0).color(egui::Color32::from_rgb(230, 160, 40)));
    }
    ui.separator();

    egui::CollapsingHeader::new("Logos").default_open(true).show(ui, |ui| {
        for a in status["assets"].as_array().cloned().unwrap_or_default() {
            ui.horizontal(|ui| {
                ui.label(a["label"].as_str().unwrap_or_default());
                let ok = a["exists"] == true;
                if ui.add_enabled(ok, egui::Button::new("Placer").small()).on_hover_text("Dans le cadre sélectionné, sinon au centre de la vue ; lié au fichier du Brand Kit").clicked() {
                    place(app, "brand.placeLogo", "asset", a["id"].as_str().unwrap_or_default(), (200.0, 120.0), "Logo placé (lié).");
                }
                if !ok {
                    ui.label(egui::RichText::new("introuvable").size(10.5).color(t.text_dim));
                }
            });
        }
    });
    egui::CollapsingHeader::new("Mockups").default_open(true).show(ui, |ui| {
        let scenes = status["scenes"].as_array().cloned().unwrap_or_default();
        if scenes.is_empty() {
            ui.label(egui::RichText::new("Aucune scène dans Mockup Studio.").size(11.0).color(t.text_dim));
        }
        for sc in scenes {
            ui.horizontal(|ui| {
                ui.label(sc["name"].as_str().unwrap_or_default());
                let ok = sc["exists"] == true;
                if ui.add_enabled(ok, egui::Button::new("Placer").small()).clicked() {
                    place(app, "brand.placeMockup", "scene", sc["id"].as_str().unwrap_or_default(), (320.0, 200.0), "Mockup placé (lié au rendu).");
                }
                if !ok {
                    ui.label(egui::RichText::new("jamais rendue").size(10.5).color(t.text_dim));
                }
            });
        }
    });
    egui::CollapsingHeader::new("Blocs").default_open(true).show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Palette").on_hover_text("Une case par couleur, avec ses valeurs (remplace la palette de la planche)").clicked() {
                place(app, "brand.placePalette", "kind", "palette", (480.0, 260.0), "Bloc palette placé.");
            }
            if ui.button("Typographies").on_hover_text("Chaque famille écrite dans sa police, avec son usage").clicked() {
                place(app, "brand.placeTypes", "kind", "types", (480.0, 260.0), "Bloc typographies placé.");
            }
        });
    });
    egui::CollapsingHeader::new("Variables").default_open(false).show(ui, |ui| {
        ui.label(egui::RichText::new("Insérées au curseur de texte ; leur valeur suit la marque.").size(11.0).color(t.text_dim));
        for v in status["variables"].as_array().cloned().unwrap_or_default() {
            let name = v["name"].as_str().unwrap_or_default().to_string();
            let value = v["value"].as_str().unwrap_or_default();
            if ui.add(egui::Button::new(egui::RichText::new(&name).size(11.5)).small()).on_hover_text(value).clicked() {
                let r = app.run("variables.insert", json!({"name": name}));
                report(app, r.map(|_| json!({})), "Variable insérée.");
            }
        }
    });
}

fn link(app: &mut DesignApp) {
    let Some(pick) = app.services.pick_open.as_mut() else { return };
    if let Some(path) = pick("brand") {
        let r = app.run("brand.link", json!({"path": path}));
        report(app, r.map(|_| json!({})), "Projet de marque lié : ouvrez « Mettre à jour » pour appliquer ses données.");
    }
}
