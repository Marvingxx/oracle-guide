use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::Session;

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="160" viewBox="0 0 400 160"><rect width="200" height="160" fill="#C82029"/><rect x="200" width="200" height="160" fill="#1D1D1B"/></svg>"##;

fn png(path: &Path, shade: u8) {
    std::fs::write(path, oracle_guide_render::Rendered { width: 160, height: 100, pixels: vec![shade; 160 * 100 * 4] }.to_png()).unwrap();
}

/// A Brand Kit project as Oracle Brand Kit and Mockup Studio write it.
fn brand_project(root: &Path) -> PathBuf {
    let dir = root.join("Marque Été");
    std::fs::create_dir_all(dir.join("Sources")).unwrap();
    std::fs::create_dir_all(dir.join("Mockups/Scènes")).unwrap();
    std::fs::write(dir.join("Sources/logo principal.svg"), LOGO).unwrap();
    std::fs::write(dir.join("Sources/symbole.svg"), LOGO.replace("#1D1D1B", "#C82029")).unwrap();
    png(&dir.join("Mockups/Carte de visite.png"), 90);
    std::fs::write(
        dir.join("Mockups/Scènes/Carte de visite.json"),
        json!({"schemaVersion": 1, "id": "scene-1", "name": "Carte de visite", "template": "x", "templateId": "t",
               "lastRender": {"file": "Mockups/Carte de visite.png", "at": "", "width": 160, "height": 100, "sha256": "", "inputs": []}})
        .to_string(),
    )
    .unwrap();
    write_brand(&dir, "#C82029", "Rouge principal", "Sources/logo principal.svg", 3);
    dir
}

fn write_brand(dir: &Path, red: &str, red_name: &str, logo: &str, colors: usize) {
    let all = [
        json!({"id": "c-rouge", "name": red_name, "hex": red, "cmyk": {"c": 0, "m": 95, "y": 85, "k": 5, "profile": "Coated FOGRA39"}, "pantone": "186 C"}),
        json!({"id": "c-noir", "name": "Noir", "hex": "#1D1D1B"}),
        json!({"id": "c-papier", "name": "Papier", "hex": "#F4F1EA"}),
    ];
    let v = json!({
        "schemaVersion": 1, "projectId": "p", "brandName": "Marque Été", "client": "Client fictif", "version": "v1",
        "assets": [
            {"id": "logo-a", "role": "primaryLogo", "label": "Logo principal", "variant": "color", "source": logo},
            {"id": "logo-b", "role": "symbol", "label": "", "variant": "color", "source": "Sources/symbole.svg"}
        ],
        "colors": all[..colors],
        "fonts": [{"id": "f1", "family": "Inter", "usage": "Texte courant", "weights": ["Regular", "SemiBold"]},
                  {"id": "f2", "family": "Police Absente Ex", "usage": "Titres", "weights": ["Bold"]}],
        "rules": {"clearSpace": "La hauteur du symbole", "minSize": "20 mm", "allowedBackgrounds": "Blanc, papier",
                  "forbidden": [{"text": "Ne pas déformer le logo"}]},
        "mockupScenes": ["Mockups/Scènes/Carte de visite.json"]
    });
    std::fs::write(dir.join("brand.json"), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}

fn keys(status: &Value) -> Vec<String> {
    status["changes"].as_array().unwrap().iter().map(|c| c["key"].as_str().unwrap().to_string()).collect()
}

fn var(s: &Session, name: &str) -> String {
    let d = &s.doc().unwrap().doc;
    let i = d.variable(name).unwrap_or_else(|| panic!("no variable {name}"));
    match &d.text_variables[i].kind {
        oracle_guide_doc::vars::VarKind::Custom { text } => text.clone(),
        _ => panic!("not custom"),
    }
}

fn swatch_hex(s: &Session, name: &str) -> Option<String> {
    s.doc().unwrap().doc.swatches.iter().find(|w| w.name == name).and_then(|w| match &w.value {
        oracle_guide_color::SwatchValue::Color { color, .. } => Some(color.to_hex()),
        _ => None,
    })
}

fn link_of(s: &Session, frame: u64) -> String {
    let d = &s.doc().unwrap().doc;
    let it = d.item(oracle_guide_doc::ItemId(frame)).unwrap();
    match &it.content {
        oracle_guide_doc::Content::Graphic(g) => d.assets[&g.asset].link.clone().unwrap(),
        _ => panic!("no graphic"),
    }
}

#[test]
fn a_guide_follows_its_brand_only_when_asked() {
    let root = std::env::temp_dir().join(format!("og-brand-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let brand = brand_project(&root);
    let mut s = Session::new();
    s.execute("file.new", &json!({"preset": "A4", "pages": 8, "facingPages": false})).unwrap();

    // Linking changes nothing in the layout; the status lists what would.
    s.execute("brand.link", &json!({"path": brand.display().to_string()})).unwrap();
    assert!(s.doc().unwrap().doc.variable("Marque · Nom").is_none());
    let st = s.execute("brand.status", &json!({})).unwrap();
    assert_eq!(st["brandName"], "Marque Été");
    let k = keys(&st);
    assert!(k.contains(&"text:Marque · Nom".to_string()) && k.contains(&"color:c-rouge".to_string()), "{k:?}");

    // Sync: variables, swatches in the « Marque » group.
    let r = s.execute("brand.sync", &json!({})).unwrap();
    assert!(r["warnings"].as_array().unwrap().is_empty(), "{r}");
    assert_eq!(var(&s, "Marque · Nom"), "Marque Été");
    assert_eq!(var(&s, "Couleur 1 · HEX"), "#C82029");
    assert_eq!(var(&s, "Couleur 1 · CMJN"), "C0 M95 J85 N5");
    assert_eq!(var(&s, "Couleur 1 · Profil CMJN"), "Coated FOGRA39");
    assert_eq!(var(&s, "Couleur 2 · Pantone"), "—", "an empty value shows a dash");
    assert_eq!(var(&s, "Couleur 1 · Pantone"), "186 C");
    assert_eq!(var(&s, "Typo 2 · Famille"), "Police Absente Ex");
    assert_eq!(var(&s, "Interdit 1"), "Ne pas déformer le logo");
    assert_eq!(swatch_hex(&s, "Rouge principal").as_deref(), Some("#c82029"));
    let groups = &s.doc().unwrap().doc.color_groups;
    assert_eq!(groups.iter().find(|g| g.name == "Marque").unwrap().swatches, ["Rouge principal", "Noir", "Papier"]);

    // Blocks: a linked logo, a linked mockup, the palette, the type block.
    let logo = s.execute("brand.placeLogo", &json!({"asset": "logo-a", "spread": 0, "rect": [60, 60, 360, 200]})).unwrap()["id"].as_u64().unwrap();
    let mock = s.execute("brand.placeMockup", &json!({"scene": "scene-1", "spread": 1, "rect": [60, 60, 400, 300]})).unwrap()["id"].as_u64().unwrap();
    assert!(link_of(&s, logo).ends_with("logo principal.svg"));
    assert!(link_of(&s, mock).ends_with("Carte de visite.png"));
    let pal = s.execute("brand.placePalette", &json!({"spread": 2, "rect": [40, 80, 555, 400]})).unwrap();
    assert_eq!(pal["ids"].as_array().unwrap().len(), 6);
    let types = s.execute("brand.placeTypes", &json!({"spread": 3, "rect": [40, 80, 555, 400]})).unwrap();
    assert_eq!(types["missingFonts"], json!(["Police Absente Ex"]), "a missing font is reported, not hidden");
    // Placing the palette again replaces it.
    s.execute("brand.placePalette", &json!({"spread": 2, "rect": [40, 80, 555, 400]})).unwrap();
    let on_spread = s.doc().unwrap().doc.spreads[2].items.iter().filter(|i| i.label == super::PALETTE_LABEL).count();
    assert_eq!(on_spread, 6);
    // A free block, edited by the designer, is not the brand's.
    s.execute("frame.create", &json!({"spread": 4, "rect": [40, 40, 500, 200], "content": "text", "text": "Notre concept, en mes mots."})).unwrap();
    assert!(keys(&s.execute("brand.status", &json!({})).unwrap()).is_empty(), "in sync");

    // The Brand Kit changes: a colour renamed and changed, the logo file edited, a colour removed.
    write_brand(&brand, "#B01D25", "Rouge Marque", "Sources/logo principal.svg", 2);
    std::fs::write(brand.join("Sources/logo principal.svg"), LOGO.replace("#1D1D1B", "#2E86AB")).unwrap();
    let st = s.execute("brand.status", &json!({})).unwrap();
    let k = keys(&st);
    for want in ["color:c-rouge", "text:Couleur 1 · Nom", "text:Couleur 1 · HEX", "text:Couleur 3 · Nom", &format!("logo:{logo}")] {
        assert!(k.contains(&want.to_string()), "{want} in {k:?}");
    }
    // Nothing was applied by looking.
    assert_eq!(var(&s, "Couleur 1 · HEX"), "#C82029");
    // Apply only the colour first, then the rest.
    s.execute("brand.sync", &json!({"keys": ["color:c-rouge"]})).unwrap();
    assert_eq!(swatch_hex(&s, "Rouge Marque").as_deref(), Some("#b01d25"));
    assert!(swatch_hex(&s, "Rouge principal").is_none(), "renamed, uses follow");
    assert_eq!(var(&s, "Couleur 1 · HEX"), "#C82029", "not picked");
    s.execute("brand.sync", &json!({})).unwrap();
    assert_eq!(var(&s, "Couleur 1 · Nom"), "Rouge Marque");
    assert_eq!(var(&s, "Couleur 3 · Nom"), "", "a colour gone from the brand empties its variables");
    assert_eq!(var(&s, "Couleur 3 · Pantone"), "");
    let links = s.execute("links.list", &json!({})).unwrap();
    assert!(links.as_array().unwrap().iter().all(|l| l["status"] == "ok"), "{links}");

    // Undo takes the whole sync back.
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(var(&s, "Couleur 1 · Nom"), "Rouge principal");
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(var(&s, "Couleur 1 · Nom"), "Rouge Marque");

    // The Brand Kit points the role at another file: the frame relinks.
    write_brand(&brand, "#B01D25", "Rouge Marque", "Sources/symbole.svg", 2);
    let k = keys(&s.execute("brand.status", &json!({})).unwrap());
    assert!(k.contains(&format!("logo:{logo}")), "{k:?}");
    s.execute("brand.sync", &json!({})).unwrap();
    assert!(link_of(&s, logo).ends_with("symbole.svg"));

    // Saved, moved with its brand, reopened: the link is found again by its relative path.
    let guide = root.join("Guide").join("guide.oracle-guide");
    std::fs::create_dir_all(guide.parent().unwrap()).unwrap();
    s.execute("file.save", &json!({"path": guide.display().to_string()})).unwrap();
    s.execute("brand.sync", &json!({})).unwrap();
    s.execute("file.save", &json!({})).unwrap();
    let moved = std::env::temp_dir().join(format!("og-brand-moved-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&moved);
    std::fs::rename(&root, &moved).unwrap();
    let mut s2 = Session::new();
    s2.execute("file.open", &json!({"path": moved.join("Guide/guide.oracle-guide").display().to_string()})).unwrap();
    let st = s2.execute("brand.status", &json!({})).unwrap();
    assert_eq!(st["linked"], true);
    assert!(st["path"].as_str().unwrap().starts_with(&moved.display().to_string()), "{st}");
    let _ = std::fs::remove_dir_all(&moved);
}

#[test]
fn a_broken_brand_is_a_clear_error() {
    let root = std::env::temp_dir().join(format!("og-brand-bad-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let e = s.execute("brand.link", &json!({"path": root.display().to_string()})).unwrap_err().to_string();
    assert!(e.contains("brand.json"), "{e}");
    std::fs::write(root.join("brand.json"), "{pas du json").unwrap();
    let e = s.execute("brand.link", &json!({"path": root.display().to_string()})).unwrap_err().to_string();
    assert!(e.contains("illisible"), "{e}");
    assert!(s.execute("brand.sync", &json!({})).is_err(), "nothing linked");
    assert_eq!(s.execute("brand.status", &json!({})).unwrap()["linked"], false);
    let _ = std::fs::remove_dir_all(&root);
}

/// The spec's guide: eight pages, linked palette, logos and a mockup, one block freely edited;
/// a readable PDF with no text cut off; saved, reopened and exported again; then the brand
/// changes and the guide follows on request.
#[test]
fn an_eight_page_guide_exports_a_readable_pdf_and_follows_the_brand() {
    let root = std::env::temp_dir().join(format!("og-guide8-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let brand = brand_project(&root);
    let mut s = Session::new();
    s.execute("file.new", &json!({"preset": "A4", "pages": 8, "facingPages": false, "margins": 40, "title": "Charte Marque Été"})).unwrap();
    s.execute("brand.link", &json!({"path": brand.join("brand.json").display().to_string()})).unwrap();
    s.execute("brand.sync", &json!({})).unwrap();
    let text = |s: &mut Session, spread: usize, rect: [f64; 4], t: &str| {
        s.execute("brand.text", &json!({"spread": spread, "rect": rect, "text": t})).unwrap()["story"].as_u64().unwrap()
    };
    // 1 Cover, 2 contents, 3 concept (free), 4 main logo and rules, 5 variants, 6 palette,
    // 7 type, 8 uses and mockup.
    text(&mut s, 0, [40.0, 80.0, 555.0, 160.0], "{Marque · Nom}");
    text(&mut s, 0, [40.0, 170.0, 555.0, 210.0], "Charte graphique {Marque · Version} — {Marque · Client}");
    s.execute("brand.placeLogo", &json!({"asset": "logo-a", "spread": 0, "rect": [140, 300, 455, 520]})).unwrap();
    s.execute("frame.create", &json!({"spread": 1, "rect": [40, 80, 555, 400], "content": "text", "text": "Sommaire\n1. Concept\n2. Logo\n3. Variantes\n4. Couleurs\n5. Typographies\n6. Usages et mises en situation"})).unwrap();
    let free = s.execute("frame.create", &json!({"spread": 2, "rect": [40, 80, 555, 400], "content": "text", "text": "Concept"})).unwrap();
    let free_story = free["story"].as_u64().unwrap();
    // The designer writes their own text in the block.
    let len = "Concept".chars().count();
    s.execute("text.select", &json!({"story": free_story, "anchor": len, "focus": len})).unwrap();
    s.execute("text.insert", &json!({"text": "\nUne marque fictive, écrite par le designer pour ce test."})).unwrap();
    s.execute("brand.placeLogo", &json!({"asset": "logo-a", "spread": 3, "rect": [100, 120, 495, 400]})).unwrap();
    text(&mut s, 3, [40.0, 440.0, 555.0, 560.0], "Zone de protection : {Marque · Zone de protection}\nTaille minimale : {Marque · Taille minimale}\nFonds : {Marque · Fonds autorisés}");
    s.execute("brand.placeLogo", &json!({"asset": "logo-b", "spread": 4, "rect": [150, 150, 445, 400]})).unwrap();
    s.execute("brand.placePalette", &json!({"spread": 5, "rect": [40, 120, 555, 600]})).unwrap();
    s.execute("brand.placeTypes", &json!({"spread": 6, "rect": [40, 120, 555, 600]})).unwrap();
    text(&mut s, 7, [40.0, 80.0, 555.0, 140.0], "À éviter : {Interdit 1}");
    s.execute("brand.placeMockup", &json!({"scene": "scene-1", "spread": 7, "rect": [40, 180, 555, 500]})).unwrap();
    assert!(keys(&s.execute("brand.status", &json!({})).unwrap()).is_empty());

    let export = |s: &mut Session| -> Vec<String> {
        let r = s.execute("file.exportPdf", &json!({})).unwrap();
        assert_eq!(r["pages"], 8);
        let bytes = super::super::base64_decode(r["base64"].as_str().unwrap());
        let images = pdf_images(&bytes);
        assert!(images[7] >= 1, "the mockup render is on page 8: {images:?}");
        crate::tests::pdf_text(&bytes)
    };
    // No text is cut off.
    let no_overset = |s: &mut Session| {
        let stories: Vec<u64> = s.doc().unwrap().doc.stories.keys().map(|k| k.0).collect();
        for st in stories {
            let g = s.execute("story.get", &json!({"story": st})).unwrap();
            assert!(g["overset"].is_null(), "story {st} overset: {}", g["text"]);
        }
    };
    no_overset(&mut s);
    let pages = export(&mut s);
    assert_eq!(pages.len(), 8);
    let compact = |p: &str| p.split_whitespace().collect::<String>();
    assert!(compact(&pages[0]).contains("MarqueÉté"), "{:?}", pages[0]);
    assert!(compact(&pages[0]).contains("v1—Clientfictif"), "{:?}", pages[0]);
    assert!(compact(&pages[2]).contains("Unemarquefictive"), "{:?}", pages[2]);
    assert!(compact(&pages[3]).contains("20mm"), "{:?}", pages[3]);
    assert!(compact(&pages[5]).contains("#C82029") && compact(&pages[5]).contains("Papier"), "{:?}", pages[5]);
    assert!(compact(&pages[6]).contains("Inter") && compact(&pages[6]).contains("Textecourant"), "{:?}", pages[6]);
    assert!(compact(&pages[7]).contains("Nepasdéformerlelogo"), "{:?}", pages[7]);

    // For print: PDF/X-4 with bleed and crop marks, with no conformance issue (the logos' and
    // swatches' RGB is colour-managed as sRGB under the CMYK output intent).
    let print = s.execute("file.exportPdf", &json!({"standard": "x4", "bleed": true, "marks": true})).unwrap();
    assert_eq!(print["pages"], 8);
    let issues: Vec<&Value> = print["warnings"].as_array().unwrap().iter().filter(|w| w.as_str().is_some_and(|w| w.starts_with("PDF/X-4"))).collect();
    assert!(issues.is_empty(), "{issues:?}");
    let bytes = super::super::base64_decode(print["base64"].as_str().unwrap());
    assert!(oracle_guide_pdf::check_pdfx4(&bytes).is_empty());

    // Saved, closed, reopened: same pages, same links, same text.
    let file = root.join("Charte.oracle-guide");
    s.execute("file.save", &json!({"path": file.display().to_string()})).unwrap();
    let mut s2 = Session::new();
    s2.execute("file.open", &json!({"path": file.display().to_string()})).unwrap();
    assert_eq!(s2.doc().unwrap().doc.spreads.len(), 8);
    assert!(keys(&s2.execute("brand.status", &json!({})).unwrap()).is_empty(), "reopened in sync");
    let again = export(&mut s2);
    assert_eq!(again, pages, "the reopened guide exports the same text");

    // The brand changes: nothing moves until the designer synchronises; then the PDF follows,
    // and the free block keeps the designer's words.
    write_brand(&brand, "#B01D25", "Rouge Marque", "Sources/logo principal.svg", 3);
    let before = export(&mut s2);
    assert!(compact(&before[5]).contains("#C82029"));
    s2.execute("brand.sync", &json!({})).unwrap();
    no_overset(&mut s2);
    let after = export(&mut s2);
    assert!(compact(&after[5]).contains("#B01D25") && compact(&after[5]).contains("RougeMarque"), "{:?}", after[5]);
    assert!(compact(&after[2]).contains("Unemarquefictive"));
    let _ = std::fs::remove_dir_all(&root);
}

/// A guide made for one brand serves as the designer's template for another: relinked, its logo
/// frames find the new brand's masters by role, its palette swatches take the new colours by
/// position, its variables the new values; free text stays.
#[test]
fn a_guide_reused_as_a_template_for_another_brand() {
    let root = std::env::temp_dir().join(format!("og-brand-template-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let a = brand_project(&root);
    let mut s = Session::new();
    s.execute("file.new", &json!({"preset": "A4", "pages": 2, "facingPages": false})).unwrap();
    s.execute("brand.link", &json!({"path": a.display().to_string()})).unwrap();
    s.execute("brand.sync", &json!({})).unwrap();
    let logo = s.execute("brand.placeLogo", &json!({"asset": "logo-a", "spread": 0, "rect": [60, 60, 360, 200]})).unwrap()["id"].as_u64().unwrap();
    s.execute("brand.placePalette", &json!({"spread": 1, "rect": [40, 80, 555, 400]})).unwrap();
    s.execute("frame.create", &json!({"spread": 0, "rect": [40, 400, 500, 500], "content": "text", "text": "Mon texte de présentation."})).unwrap();

    // Another brand: other ids, other colours, its own main logo.
    let b = root.join("Autre Marque");
    std::fs::create_dir_all(&b).unwrap();
    std::fs::write(b.join("logo.svg"), LOGO.replace("#C82029", "#1B5E20")).unwrap();
    std::fs::write(b.join("brand.json"), json!({
        "schemaVersion": 1, "projectId": "q", "brandName": "Autre Marque", "version": "v2",
        "assets": [{"id": "x1", "role": "primaryLogo", "label": "Logo", "source": "logo.svg"}],
        "colors": [{"id": "b1", "name": "Vert forêt", "hex": "#1B5E20"}, {"id": "b2", "name": "Sable", "hex": "#E8DCC4"}]
    }).to_string()).unwrap();
    s.execute("brand.link", &json!({"path": b.display().to_string()})).unwrap();
    let k = keys(&s.execute("brand.status", &json!({})).unwrap());
    assert!(k.contains(&format!("logo:{logo}")) && k.contains(&"color:b1".to_string()), "{k:?}");
    let r = s.execute("brand.sync", &json!({})).unwrap();
    assert!(r["warnings"].as_array().unwrap().is_empty(), "{r}");

    assert!(link_of(&s, logo).ends_with("logo.svg") && link_of(&s, logo).contains("Autre Marque"), "{}", link_of(&s, logo));
    let d = &s.doc().unwrap().doc;
    assert_eq!(d.item(oracle_guide_doc::ItemId(logo)).unwrap().label, "oracle:logo:x1@primaryLogo");
    // The palette's first two cases now show the new colours, under the new names.
    let fills: Vec<String> = d.spreads[1].items.iter().filter(|i| i.label == super::PALETTE_LABEL && i.text_frame().is_none()).map(|i| i.fill.swatch.clone()).collect();
    assert_eq!(&fills[..2], ["Vert forêt", "Sable"]);
    assert_eq!(swatch_hex(&s, "Vert forêt").as_deref(), Some("#1b5e20"));
    assert_eq!(var(&s, "Marque · Nom"), "Autre Marque");
    assert_eq!(var(&s, "Couleur 3 · Nom"), "", "the old brand's third colour is emptied");
    let texts: Vec<String> = s.doc().unwrap().doc.stories.values().map(|st| st.text.to_string()).collect();
    assert!(texts.iter().any(|t| t.contains("Mon texte de présentation.")));
    assert!(keys(&s.execute("brand.status", &json!({})).unwrap()).is_empty(), "in sync with the new brand");
    let _ = std::fs::remove_dir_all(&root);
}

/// Images drawn per page, through hayro's interpreter.
fn pdf_images(bytes: &[u8]) -> Vec<usize> {
    use hayro_interpret::font::Glyph;
    use hayro_interpret::{BlendMode, ClipPath, Context, Device, GlyphDrawMode, Image, InterpreterCache, InterpreterSettings, Paint, PathDrawMode, SoftMask, interpret_page};
    struct Count(usize);
    impl Device<'_> for Count {
        fn set_soft_mask(&mut self, _: Option<SoftMask<'_>>) {}
        fn set_blend_mode(&mut self, _: BlendMode) {}
        fn draw_path(&mut self, _: &kurbo::BezPath, _: kurbo::Affine, _: &Paint<'_>, _: &PathDrawMode) {}
        fn push_clip_path(&mut self, _: &ClipPath) {}
        fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: BlendMode) {}
        fn draw_glyph(&mut self, _: &Glyph<'_>, _: kurbo::Affine, _: kurbo::Affine, _: &Paint<'_>, _: &GlyphDrawMode) {}
        fn draw_image(&mut self, _: Image<'_, '_>, _: kurbo::Affine) {
            self.0 += 1;
        }
        fn pop_clip_path(&mut self) {}
        fn pop_transparency_group(&mut self) {}
    }
    let pdf = hayro_syntax::Pdf::new(bytes.to_vec()).expect("valid PDF");
    let cache = InterpreterCache::new();
    pdf.pages()
        .iter()
        .map(|page| {
            let mut ctx = Context::new(kurbo::Affine::IDENTITY, kurbo::Rect::new(0.0, 0.0, 1.0, 1.0), &cache, pdf.xref(), InterpreterSettings::default());
            let mut c = Count(0);
            interpret_page(page, &mut ctx, &mut c);
            c.0
        })
        .collect()
}
