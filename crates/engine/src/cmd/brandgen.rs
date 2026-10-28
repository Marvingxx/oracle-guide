//! Oracle: whole documents generated from a Brand Kit project — the brand guidelines and the logo
//! presentation — and the brand grid graphics and bento layouts they are made of.
//!
//! Every page is ordinary layout built through the existing commands: text with brand variables,
//! logos linked and labelled `oracle:logo:` (so `brand.sync` keeps following the Brand Kit),
//! the palette and type blocks, and grid graphics drawn by `oracle-brandtools` from the logo's own
//! outlines, written as SVG files in the project's `Grilles` folder and placed linked, labelled
//! `oracle:grid:<asset>@<kind>`. The designer edits anything afterwards; nothing is locked.

use std::path::{Path, PathBuf};

use oracle_brandtools as bt;
use serde_json::{Value, json};

use super::brand::{self, Brand, linked_brand, new_frame, swatch_for};
use super::{CommandSpec, always, bad, cmd, has_doc, str_param};
use crate::{Result, Session};

/// Label of generated grid graphics and bento tiles.
pub const GRID_LABEL: &str = "oracle:grid:";
pub const BENTO_LABEL: &str = "oracle:bento";
/// The largest logo master read to draw a grid.
const MAX_SVG: u64 = 16 << 20;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "brand.generateGuide",
            "Générer la charte graphique…",
            ["Layout", "Brand"],
            None,
            "{path?: brand.json or its folder (default: the brand the active document follows), size?: [width, height] (pt; default A4 landscape 842 × 595), sections?: [cover, contents, logo, variants, backgrounds, clearspace, construction, lockups, colors, typography, rules, mockups, end] (default: every section the brand has material for)} — a new document linked to the brand, one page per section → {pages, sections, warnings}",
            always,
            generate_guide
        ),
        cmd!(
            "brand.generatePresentation",
            "Générer la présentation de pistes…",
            ["Layout", "Brand"],
            None,
            "{path?: brand.json or its folder (default: the brand the active document follows), concepts?: [asset ids] (1-3; default the brand's logos by role, up to 3), studio?: {name, contact}, size?: [w, h] (default 1280 × 720)} — a new document: a cover, then for each concept its own pages (the logo, on colour, its construction and clearspace, a bento board), and a closing page; the studio on every page → {pages, concepts, warnings}",
            always,
            generate_presentation
        ),
        cmd!(
            "brand.gridGraphic",
            "Placer une grille de logo",
            [],
            None,
            "{asset: logo id (an SVG master), kind: construction|clearspace|lockupHorizontal|lockupVertical|lockupCondensed, params?: oracle-brandtools settings for the kind, spread?, rect} — the grid drawn from the logo's outlines, written to <project>/Grilles and placed linked, labelled oracle:grid:<asset>@<kind> → {id, file}",
            has_doc,
            grid_graphic
        ),
        cmd!(
            "brand.bento",
            "Grille bento",
            ["Object"],
            None,
            "{spread?, rect, params?: {style: simple|centered|brick|random, sections, radius, gutter, seed}, brandColors?: bool (fill the tiles with the brand's swatches in turn)} — graphic frames tiling the rectangle, labelled oracle:bento (replaces the spread's previous bento in that rectangle) → {ids}",
            has_doc,
            bento
        ),
    ]
}

// --- Grid graphics ------------------------------------------------------------------------------

fn read_logo(file: &Path) -> std::result::Result<bt::Logo, String> {
    let is_svg = file.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    if !is_svg {
        return Err(format!("{} : les grilles se calculent sur un maître SVG", file.display()));
    }
    let meta = crate::io::metadata(file).map_err(|e| format!("{} : {e}", file.display()))?;
    if meta.len() > MAX_SVG {
        return Err(format!("{} : fichier trop grand", file.display()));
    }
    let bytes = crate::io::read(file).map_err(|e| format!("{} : {e}", file.display()))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{} : SVG illisible", file.display()))?;
    bt::Logo::from_svg(&text).map_err(|e| format!("{} : {e}", file.display()))
}

/// The grid of `kind` for `logo`, as a standalone SVG with the logo under it.
fn grid_svg(logo: &bt::Logo, kind: &str, params: &Value) -> std::result::Result<String, String> {
    fn de<T: serde::de::DeserializeOwned + Default>(v: &Value) -> std::result::Result<T, String> {
        if v.is_null() { Ok(T::default()) } else { serde_json::from_value(v.clone()).map_err(|e| format!("params : {e}")) }
    }
    let lockup = |k: bt::LockupKind| -> std::result::Result<String, String> {
        let mut p: bt::LockupParams = de(params)?;
        p.kind = k;
        let r = bt::lockup(logo, None, &p)?;
        Ok(r.drawing.to_svg(None, None))
    };
    match kind {
        "construction" => {
            let p: bt::ConstructionParams =
                if params.is_null() { bt::ConstructionParams { clean: true, measurements: true, ..Default::default() } } else { de(params)? };
            Ok(bt::construction(logo, None, &p).to_svg(Some(logo), None))
        }
        "clearspace" => {
            let p: bt::ClearspaceParams = de(params)?;
            Ok(bt::clearspace(logo, &p).0.to_svg(Some(logo), None))
        }
        "lockupHorizontal" => lockup(bt::LockupKind::Horizontal),
        "lockupVertical" => lockup(bt::LockupKind::Vertical),
        "lockupCondensed" => lockup(bt::LockupKind::Condensed),
        other => Err(format!("grille inconnue : {other}")),
    }
}

/// Writes the grid file into the project's `Grilles` folder.
fn write_grid(brand: &Brand, asset: &brand::BrandAsset, kind: &str, params: &Value) -> std::result::Result<PathBuf, String> {
    let logo = read_logo(&asset.file)?;
    let svg = grid_svg(&logo, kind, params)?;
    let dir = brand.file.parent().map(|d| d.join("Grilles")).ok_or("projet de marque sans dossier")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{} : {e}", dir.display()))?;
    let safe: String = asset.id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    let path = dir.join(format!("{safe}-{kind}.svg"));
    crate::io::write_file(&path, svg.as_bytes()).map_err(|e| format!("{} : {e}", path.display()))?;
    Ok(path)
}

/// Places `file` linked in a new graphic frame, fitted proportionally and labelled.
fn place_file(s: &mut Session, spread: &Value, rect: [f64; 4], file: &Path, label: &str) -> Result<u64> {
    let frame = new_frame(s, spread, rect, "graphic", None)?;
    s.execute("object.label", &json!({"label": label, "ids": [frame.0]}))?;
    s.execute("file.place", &json!({"path": file.display().to_string(), "frame": frame.0, "link": true}))?;
    s.execute("object.fittingOptions", &json!({"autoFit": true, "fitting": "fitProportionally", "align": 4, "ids": [frame.0]}))?;
    s.execute("object.fit", &json!({"mode": "fitProportionally", "ids": [frame.0]}))?;
    Ok(frame.0)
}

fn grid_graphic(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brand.gridGraphic";
    let Some((brand, _)) = linked_brand(s).map_err(|e| bad(C, e))? else { return Err(bad(C, "aucun projet de marque lié (brand.link)")) };
    let id = str_param(p, "asset").ok_or_else(|| bad(C, "missing asset"))?;
    let kind = str_param(p, "kind").unwrap_or("construction");
    let asset = brand.assets.iter().find(|a| a.id == id).ok_or_else(|| bad(C, format!("pas de logo {id} dans le Brand Kit")))?;
    let file = write_grid(&brand, asset, kind, p.get("params").unwrap_or(&Value::Null)).map_err(|e| bad(C, e))?;
    let rect = brand::rect_param(p, C)?;
    let frame = place_file(s, p.get("spread").unwrap_or(&Value::Null), rect, &file, &format!("{GRID_LABEL}{id}@{kind}"))?;
    Ok(json!({"id": frame, "file": file.display().to_string()}))
}

// --- Bento --------------------------------------------------------------------------------------

fn bento(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brand.bento";
    let [x0, y0, x1, y1] = brand::rect_param(p, C)?;
    let params: bt::BentoParams = match p.get("params") {
        Some(v) => serde_json::from_value(v.clone()).map_err(|e| bad(C, e.to_string()))?,
        None => bt::BentoParams::default(),
    };
    let spread = p.get("spread").cloned().unwrap_or(Value::Null);
    // A bento drawn again over the same rectangle replaces the previous one (a shuffle).
    let area = bt::Rect::new(x0, y0, x1, y1);
    let old: Vec<u64> = {
        let d = &s.doc()?.doc;
        let index = spread.as_u64().and_then(|i| usize::try_from(i).ok()).unwrap_or(0);
        d.spreads
            .get(index)
            .map(|sp| {
                sp.items
                    .iter()
                    .filter(|it| {
                        let b = it.bounds();
                        it.label == BENTO_LABEL && area.contains_rect(&bt::Rect::new(b.x0, b.y0, b.x1, b.y1), 1.0)
                    })
                    .map(|it| it.id.0)
                    .collect()
            })
            .unwrap_or_default()
    };
    if !old.is_empty() {
        s.execute("edit.clear", &json!({"ids": old}))?;
    }
    let swatches: Vec<String> = if p.get("brandColors").and_then(Value::as_bool).unwrap_or(false) {
        match linked_brand(s).map_err(|e| bad(C, e))? {
            Some((brand, link)) => (0..brand.colors.len()).filter_map(|i| swatch_for(&link, &s.doc().ok()?.doc, &brand, i)).collect(),
            None => Vec::new(),
        }
    } else {
        Vec::new()
    };
    let mut ids = Vec::new();
    for (i, (r, radius)) in bt::bento::tiles(area, &params).into_iter().enumerate() {
        let id = new_frame(s, &spread, [r.x0, r.y0, r.x1, r.y1], "graphic", None)?;
        if radius > 0.0 {
            s.execute("object.cornerOptions", &json!({"shape": "rounded", "size": radius, "ids": [id.0]}))?;
        }
        if let Some(sw) = swatches.get(i % swatches.len().max(1)) {
            s.execute("object.fill", &json!({"swatch": sw, "ids": [id.0]}))?;
        }
        ids.push(id.0);
    }
    s.execute("object.label", &json!({"label": BENTO_LABEL, "ids": ids}))?;
    Ok(json!({"ids": ids}))
}

// --- Generated documents ------------------------------------------------------------------------

/// Builds pages one after the other on a document of `n` single pages (spread i = page i).
struct Pages<'a> {
    w: f64,
    h: f64,
    brand: &'a Brand,
    font: String,
    /// The brand's colours as (swatch name, sRGB), in order.
    colors: Vec<(String, [u8; 3])>,
    /// The colours the presented logo is painted with (its SVG master's flat fills).
    logo_colors: Vec<[u8; 3]>,
    footer: String,
    warnings: Vec<String>,
}

const M: f64 = 40.0;

impl Pages<'_> {
    fn text(&self, s: &mut Session, page: usize, rect: [f64; 4], template: &str, size: f64, fill: &str) -> Result<()> {
        s.execute(
            "brand.text",
            &json!({"spread": page, "rect": rect, "text": template, "attrs": {"fontFamily": self.font, "size": size, "fill": fill}}),
        )?;
        Ok(())
    }

    fn title(&self, s: &mut Session, page: usize, number: &str, title: &str) -> Result<()> {
        self.text(s, page, [M, M, self.w - M, M + 18.0], number, 9.0, "[Black]")?;
        self.text(s, page, [M, M + 20.0, self.w - M, M + 62.0], title, 26.0, "[Black]")?;
        self.text(s, page, [M, self.h - M + 8.0, self.w - M, self.h - M + 22.0], &self.footer, 8.0, "[Black]")
    }

    fn fill_rect(&self, s: &mut Session, page: usize, rect: [f64; 4], swatch: &str) -> Result<u64> {
        let id = new_frame(s, &json!(page), rect, "unassigned", None)?;
        s.execute("object.fill", &json!({"swatch": swatch, "ids": [id.0]}))?;
        s.execute("object.stroke", &json!({"weight": 0, "ids": [id.0]})).ok();
        Ok(id.0)
    }

    fn logo(&self, s: &mut Session, page: usize, asset: &str, rect: [f64; 4]) -> Result<()> {
        s.execute("brand.placeLogo", &json!({"asset": asset, "spread": page, "rect": rect}))?;
        Ok(())
    }

    fn grid(&mut self, s: &mut Session, page: usize, asset: &brand::BrandAsset, kind: &str, rect: [f64; 4]) -> Result<()> {
        match write_grid(self.brand, asset, kind, &Value::Null) {
            Ok(file) => {
                place_file(s, &json!(page), rect, &file, &format!("{GRID_LABEL}{}@{kind}", asset.id))?;
            }
            Err(e) => self.warnings.push(e),
        }
        Ok(())
    }

    /// The content area under the title.
    fn body(&self) -> [f64; 4] {
        [M, M + 80.0, self.w - M, self.h - M - 10.0]
    }

    /// Paper, the brand's colours, then black: every background a page can use.
    fn backgrounds(&self) -> Vec<(String, [u8; 3])> {
        let mut v = vec![("[Paper]".to_string(), [255, 255, 255])];
        v.extend(self.colors.iter().cloned());
        v.push(("[Black]".to_string(), [0, 0, 0]));
        v
    }

    /// Whether the logo reads on `bg`: each colour it is painted with stands out from it.
    fn readable(&self, bg: [u8; 3]) -> bool {
        self.logo_colors.iter().all(|c| contrast(*c, bg) >= 1.6)
    }

    /// A coloured background the logo reads on: the first brand colour (not near-white) that
    /// works, else black, else paper.
    fn feature(&self) -> (String, [u8; 3]) {
        let mut colored: Vec<(String, [u8; 3])> = self.colors.iter().filter(|(_, c)| luminance(*c) < 0.85).cloned().collect();
        colored.push(("[Black]".into(), [0, 0, 0]));
        colored.into_iter().find(|(_, c)| self.readable(*c)).unwrap_or_else(|| ("[Paper]".into(), [255, 255, 255]))
    }
}

/// WCAG relative luminance of an sRGB colour.
fn luminance(c: [u8; 3]) -> f64 {
    let lin = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// The swatch text is set in on a background: paper on dark, black on light.
fn ink_on(bg: [u8; 3]) -> &'static str {
    if luminance(bg) < 0.35 { "[Paper]" } else { "[Black]" }
}

/// The flat colours of a logo's SVG master; none when it isn't SVG or can't be read.
fn logo_colors(asset: Option<&brand::BrandAsset>) -> Vec<[u8; 3]> {
    let Some(logo) = asset.and_then(|a| read_logo(&a.file).ok()) else { return Vec::new() };
    let mut out: Vec<[u8; 3]> = Vec::new();
    for shape in logo.components.iter().flat_map(|c| c.shapes.iter()) {
        let Some(hex) = shape.fill.as_deref().and_then(|h| h.strip_prefix('#')) else { continue };
        let Ok(v) = u32::from_str_radix(hex, 16) else { continue };
        let c = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// Whether the brand has material for a section.
fn available(brand: &Brand, section: &str) -> bool {
    let svg = |a: &brand::BrandAsset| a.file.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    match section {
        "logo" | "variants" | "backgrounds" => !brand.assets.is_empty(),
        "clearspace" | "construction" | "lockups" => brand.assets.iter().any(svg),
        "colors" => !brand.colors.is_empty(),
        "typography" => !brand.fonts.is_empty(),
        "rules" => !brand.forbidden.is_empty() || !brand.clear_space.is_empty() || !brand.backgrounds.is_empty(),
        "mockups" => brand.scenes.iter().any(|sc| sc.render.is_some()),
        _ => true,
    }
}

const SECTIONS: [&str; 13] = [
    "cover",
    "contents",
    "logo",
    "variants",
    "backgrounds",
    "clearspace",
    "construction",
    "lockups",
    "colors",
    "typography",
    "rules",
    "mockups",
    "end",
];

fn section_title(s: &str) -> &'static str {
    match s {
        "cover" => "Couverture",
        "contents" => "Sommaire",
        "logo" => "Logo principal",
        "variants" => "Déclinaisons du logo",
        "backgrounds" => "Logo sur les couleurs",
        "clearspace" => "Zone de protection",
        "construction" => "Construction",
        "lockups" => "Agencements",
        "colors" => "Couleurs",
        "typography" => "Typographies",
        "rules" => "Règles d'usage",
        "mockups" => "Applications",
        _ => "Contact",
    }
}

fn size_param(p: &Value, default: [f64; 2], cmd: &str) -> Result<[f64; 2]> {
    match p.get("size") {
        None | Some(Value::Null) => Ok(default),
        Some(v) => {
            let a: Vec<f64> = v.as_array().map(|a| a.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
            match a[..] {
                [w, h] if w.is_finite() && h.is_finite() && (100.0..=10_000.0).contains(&w) && (100.0..=10_000.0).contains(&h) => Ok([w, h]),
                _ => Err(bad(cmd, "size is [width, height] in points, 100 to 10000")),
            }
        }
    }
}

/// A new document of `pages` single pages, linked to the brand and synced.
/// The brand project: `path`, else the one the active document follows.
fn brand_path(s: &Session, p: &Value, cmd: &str) -> Result<String> {
    if let Some(path) = str_param(p, "path") {
        return Ok(path.to_string());
    }
    s.active()
        .and_then(|st| st.doc.settings.brand.as_ref())
        .map(|b| b.path.clone())
        .ok_or_else(|| bad(cmd, "missing path (brand.json or its folder), and the document follows no brand"))
}

fn start(s: &mut Session, path: &str, size: [f64; 2], pages: usize, title: &str) -> Result<()> {
    s.execute("file.new", &json!({"width": size[0], "height": size[1], "pages": pages, "facingPages": false, "margins": M, "title": title}))?;
    s.execute("brand.link", &json!({"path": path}))?;
    s.execute("brand.sync", &json!({}))?;
    Ok(())
}

fn linked(s: &Session, cmd: &str) -> Result<Brand> {
    let Some((brand, _)) = linked_brand(s).map_err(|e| bad(cmd, e))? else { return Err(bad(cmd, "aucun projet de marque lié")) };
    Ok(brand)
}

fn colors_of(s: &Session, brand: &Brand) -> Vec<(String, [u8; 3])> {
    let Ok(st) = s.doc() else { return Vec::new() };
    let Some(link) = st.doc.settings.brand.as_ref() else { return Vec::new() };
    (0..brand.colors.len()).filter_map(|i| Some((swatch_for(link, &st.doc, brand, i)?, brand.colors.get(i)?.rgb))).collect()
}

fn font_of(s: &mut Session, brand: &Brand) -> String {
    for f in &brand.fonts {
        if brand::font_installed(s, &f.family) {
            return f.family.clone();
        }
    }
    "Inter".into()
}

/// The brand's logos in role order: the primary logo first.
fn logos(brand: &Brand) -> Vec<&brand::BrandAsset> {
    const ORDER: [&str; 8] = ["primaryLogo", "horizontalLogo", "verticalLogo", "symbol", "secondaryLogo", "signature", "favicon", "other"];
    let mut v: Vec<&brand::BrandAsset> = brand.assets.iter().collect();
    v.sort_by_key(|a| ORDER.iter().position(|r| *r == a.role).unwrap_or(ORDER.len()));
    v
}

fn generate_guide(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brand.generateGuide";
    let [w, h] = size_param(p, [842.0, 595.0], C)?;
    // Which sections: asked for, or every one the brand has material for (read the brand first).
    let path = brand_path(s, p, C)?;
    let path = path.as_str();
    let preview = brand::load_brand(Path::new(path)).map_err(|e| bad(C, e))?;
    let asked: Vec<String> = match p.get("sections").and_then(Value::as_array) {
        Some(a) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        None => SECTIONS.iter().map(|x| x.to_string()).collect(),
    };
    if let Some(unknown) = asked.iter().find(|x| !SECTIONS.contains(&x.as_str())) {
        return Err(bad(C, format!("section inconnue : {unknown}")));
    }
    let sections: Vec<String> = asked.into_iter().filter(|x| available(&preview, x)).collect();
    if sections.is_empty() {
        return Err(bad(C, "aucune section à générer"));
    }
    start(s, path, [w, h], sections.len(), &format!("Charte {}", preview.name))?;
    let brand = linked(s, C)?;
    let logos = logos(&brand);
    let mut pg = Pages {
        w,
        h,
        colors: colors_of(s, &brand),
        logo_colors: logo_colors(logos.first().copied()),
        font: font_of(s, &brand),
        footer: "{Marque · Nom} — Charte graphique {Marque · Version}".into(),
        brand: &brand,
        warnings: Vec::new(),
    };
    let primary = logos.first().map(|a| a.id.clone());
    let svg_logo = logos.iter().find(|a| a.file.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("svg"))).copied();
    let numbered: Vec<&String> = sections.iter().filter(|x| !matches!(x.as_str(), "cover" | "contents" | "end")).collect();
    for (page, section) in sections.iter().enumerate() {
        let n = numbered.iter().position(|x| *x == section).map(|i| format!("{:02}", i + 1)).unwrap_or_default();
        let body = pg.body();
        match section.as_str() {
            "cover" => {
                let (band, band_rgb) = pg.colors.first().cloned().unwrap_or_else(|| ("[Black]".into(), [0, 0, 0]));
                let split = w * 0.42;
                pg.fill_rect(s, page, [0.0, 0.0, split, h], &band)?;
                let ink = ink_on(band_rgb);
                pg.text(s, page, [M, h * 0.62, split - M, h * 0.62 + 60.0], "{Marque · Nom}", 40.0, ink)?;
                pg.text(
                    s,
                    page,
                    [M, h * 0.62 + 64.0, split - M, h * 0.62 + 100.0],
                    "Charte graphique {Marque · Version}\n{Marque · Client}",
                    12.0,
                    ink,
                )?;
                let (bg, _) = pg.backgrounds().into_iter().find(|(_, c)| pg.readable(*c)).unwrap_or_else(|| ("[Paper]".into(), [255, 255, 255]));
                pg.fill_rect(s, page, [split, 0.0, w, h], &bg)?;
                if let Some(id) = &primary {
                    pg.logo(s, page, id, [split + M * 1.5, h * 0.3, w - M * 1.5, h * 0.7])?;
                }
            }
            "contents" => {
                pg.title(s, page, "", "Sommaire")?;
                let list: Vec<String> = numbered.iter().enumerate().map(|(i, x)| format!("{:02}   {}", i + 1, section_title(x))).collect();
                pg.text(s, page, [M, body[1], w * 0.6, body[3]], &list.join("\n"), 14.0, "[Black]")?;
            }
            "logo" => {
                pg.title(s, page, &n, "Logo principal")?;
                if let Some(id) = &primary {
                    pg.logo(s, page, id, [w * 0.2, body[1] + 20.0, w * 0.8, body[3] - 70.0])?;
                }
                pg.text(
                    s,
                    page,
                    [M, body[3] - 50.0, w - M, body[3]],
                    "Zone de protection : {Marque · Zone de protection}\nTaille minimale : {Marque · Taille minimale}",
                    10.0,
                    "[Black]",
                )?;
            }
            "variants" => {
                pg.title(s, page, &n, "Déclinaisons du logo")?;
                let shown: Vec<&&brand::BrandAsset> = logos.iter().take(8).collect();
                let cols = shown.len().clamp(1, 4);
                let rows = shown.len().div_ceil(cols).max(1);
                let (cw, ch) = ((body[2] - body[0]) / cols as f64, (body[3] - body[1]) / rows as f64);
                for (i, a) in shown.iter().enumerate() {
                    let (x, y) = (body[0] + (i % cols) as f64 * cw, body[1] + (i / cols) as f64 * ch);
                    pg.logo(s, page, &a.id, [x + 12.0, y + 8.0, x + cw - 12.0, y + ch - 30.0])?;
                    pg.text(s, page, [x + 12.0, y + ch - 24.0, x + cw - 12.0, y + ch - 8.0], &a.label.replace(['{', '}'], ""), 9.0, "[Black]")?;
                }
            }
            "backgrounds" => {
                pg.title(s, page, &n, "Logo sur les couleurs")?;
                let fills: Vec<String> = pg.backgrounds().into_iter().filter(|(_, c)| pg.readable(*c)).map(|(sw, _)| sw).take(6).collect();
                if fills.is_empty() {
                    pg.warnings.push("aucun fond de la palette ne contraste assez avec le logo".into());
                }
                let cols = fills.len().clamp(1, 3);
                let rows = fills.len().div_ceil(cols);
                let gap = 10.0;
                let (cw, ch) =
                    ((body[2] - body[0] - gap * (cols as f64 - 1.0)) / cols as f64, (body[3] - body[1] - gap * (rows as f64 - 1.0)) / rows as f64);
                for (i, f) in fills.iter().enumerate() {
                    let (x, y) = (body[0] + (i % cols) as f64 * (cw + gap), body[1] + (i / cols) as f64 * (ch + gap));
                    let tile = pg.fill_rect(s, page, [x, y, x + cw, y + ch], f)?;
                    if f == "[Paper]" {
                        s.execute("object.stroke", &json!({"weight": 0.5, "swatch": "[Black]", "ids": [tile]})).ok();
                    }
                    if let Some(id) = &primary {
                        pg.logo(s, page, id, [x + cw * 0.2, y + ch * 0.2, x + cw * 0.8, y + ch * 0.8])?;
                    }
                }
            }
            "clearspace" => {
                pg.title(s, page, &n, "Zone de protection")?;
                if let Some(a) = svg_logo {
                    pg.grid(s, page, a, "clearspace", [body[0], body[1], body[2], body[3] - 40.0])?;
                }
                pg.text(
                    s,
                    page,
                    [M, body[3] - 34.0, w - M, body[3]],
                    "Aucun élément ne pénètre la zone de protection. {Marque · Zone de protection}",
                    10.0,
                    "[Black]",
                )?;
            }
            "construction" => {
                pg.title(s, page, &n, "Construction")?;
                if let Some(a) = svg_logo {
                    pg.grid(s, page, a, "construction", body)?;
                }
            }
            "lockups" => {
                pg.title(s, page, &n, "Agencements")?;
                if let Some(a) = svg_logo {
                    let mid = (body[0] + body[2]) / 2.0;
                    pg.grid(s, page, a, "lockupHorizontal", [body[0], body[1], mid - 10.0, body[3]])?;
                    pg.grid(s, page, a, "lockupVertical", [mid + 10.0, body[1], body[2], body[3]])?;
                }
            }
            "colors" => {
                pg.title(s, page, &n, "Couleurs")?;
                s.execute("brand.placePalette", &json!({"spread": page, "rect": body}))?;
            }
            "typography" => {
                pg.title(s, page, &n, "Typographies")?;
                s.execute("brand.placeTypes", &json!({"spread": page, "rect": body}))?;
            }
            "rules" => {
                pg.title(s, page, &n, "Règles d'usage")?;
                let mut t = String::from("Fonds autorisés : {Marque · Fonds autorisés}\n\nÀ éviter :");
                for i in 1..=brand.forbidden.len().min(8) {
                    t.push_str(&format!("\n— {{Interdit {i}}}"));
                }
                pg.text(s, page, [body[0], body[1], body[2], body[3]], &t, 13.0, "[Black]")?;
            }
            "mockups" => {
                pg.title(s, page, &n, "Applications")?;
                let scenes: Vec<String> = brand.scenes.iter().filter(|sc| sc.render.is_some()).take(2).map(|sc| sc.id.clone()).collect();
                let k = scenes.len().max(1) as f64;
                let cw = (body[2] - body[0] - 12.0 * (k - 1.0)) / k;
                for (i, id) in scenes.iter().enumerate() {
                    let x = body[0] + i as f64 * (cw + 12.0);
                    s.execute("brand.placeMockup", &json!({"scene": id, "spread": page, "rect": [x, body[1], x + cw, body[3]]}))?;
                }
            }
            _ => {
                let (bg, rgb) = pg.colors.first().cloned().unwrap_or_else(|| ("[Black]".into(), [0, 0, 0]));
                pg.fill_rect(s, page, [0.0, 0.0, w, h], &bg)?;
                pg.text(s, page, [M, h * 0.42, w - M, h * 0.42 + 40.0], "{Marque · Nom}", 28.0, ink_on(rgb))?;
                pg.text(
                    s,
                    page,
                    [M, h * 0.42 + 44.0, w - M, h * 0.42 + 64.0],
                    "Charte graphique {Marque · Version} — {Marque · Client}",
                    11.0,
                    ink_on(rgb),
                )?;
            }
        }
    }
    s.execute("layout.selectPage", &json!({"page": 0})).ok();
    Ok(json!({"pages": sections.len(), "sections": sections, "warnings": pg.warnings}))
}

fn generate_presentation(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brand.generatePresentation";
    let [w, h] = size_param(p, [1280.0, 720.0], C)?;
    let path = brand_path(s, p, C)?;
    let path = path.as_str();
    let preview = brand::load_brand(Path::new(path)).map_err(|e| bad(C, e))?;
    let concepts: Vec<String> = match p.get("concepts").and_then(Value::as_array) {
        Some(a) => a.iter().filter_map(Value::as_str).map(str::to_string).take(3).collect(),
        None => logos(&preview).iter().take(3).map(|a| a.id.clone()).collect(),
    };
    if concepts.is_empty() {
        return Err(bad(C, "le Brand Kit n'a aucun logo à présenter"));
    }
    if let Some(missing) = concepts.iter().find(|id| !preview.assets.iter().any(|a| &a.id == *id)) {
        return Err(bad(C, format!("pas de logo {missing} dans le Brand Kit")));
    }
    let studio = p.get("studio").cloned().unwrap_or(Value::Null);
    let studio_name = studio.get("name").and_then(Value::as_str).unwrap_or("").replace(['{', '}'], "");
    let contact = studio.get("contact").and_then(Value::as_str).unwrap_or("").replace(['{', '}'], "");
    // Cover, then six pages a concept, then the closing page.
    const PER: usize = 6;
    let count = 2 + concepts.len() * PER;
    start(s, path, [w, h], count, &format!("Présentation {}", preview.name))?;
    let brand = linked(s, C)?;
    let footer = if studio_name.is_empty() { "{Marque · Nom}".to_string() } else { format!("{studio_name} — {{Marque · Nom}}") };
    let mut pg =
        Pages { w, h, colors: colors_of(s, &brand), logo_colors: Vec::new(), font: font_of(s, &brand), footer, brand: &brand, warnings: Vec::new() };
    // Cover.
    let (cover, cover_rgb) = pg.colors.first().cloned().unwrap_or_else(|| ("[Black]".into(), [0, 0, 0]));
    let ink = ink_on(cover_rgb);
    pg.fill_rect(s, 0, [0.0, 0.0, w, h], &cover)?;
    pg.text(s, 0, [M * 2.0, h * 0.4, w - M * 2.0, h * 0.4 + 70.0], "{Marque · Nom}", 54.0, ink)?;
    let sub = format!("Pistes de logo — {} proposition{}", concepts.len(), if concepts.len() > 1 { "s" } else { "" });
    pg.text(s, 0, [M * 2.0, h * 0.4 + 76.0, w - M * 2.0, h * 0.4 + 100.0], &sub, 16.0, ink)?;
    if !studio_name.is_empty() {
        pg.text(s, 0, [M * 2.0, h - M * 2.0, w - M * 2.0, h - M * 2.0 + 20.0], &studio_name, 12.0, ink)?;
    }
    for (k, id) in concepts.iter().enumerate() {
        let Some(asset) = brand.assets.iter().find(|a| &a.id == id) else { continue };
        pg.logo_colors = logo_colors(Some(asset));
        let base = 1 + k * PER;
        let num = format!("Piste {}", k + 1);
        let body = pg.body();
        // 1. The concept's opening page.
        pg.fill_rect(s, base, [0.0, 0.0, w, h], "[Black]")?;
        pg.text(s, base, [M * 2.0, h * 0.42, w - M * 2.0, h * 0.42 + 60.0], &num, 48.0, "[Paper]")?;
        pg.text(s, base, [M * 2.0, h * 0.42 + 64.0, w - M * 2.0, h * 0.42 + 86.0], &asset.label.replace(['{', '}'], ""), 16.0, "[Paper]")?;
        // 2. The logo, alone.
        pg.title(s, base + 1, &num, "Le logo")?;
        pg.logo(s, base + 1, id, [w * 0.25, body[1] + 10.0, w * 0.75, body[3] - 10.0])?;
        // 3. On a brand colour it reads on.
        let (feature, _) = pg.feature();
        pg.fill_rect(s, base + 2, [0.0, 0.0, w, h], &feature)?;
        pg.logo(s, base + 2, id, [w * 0.25, h * 0.2, w * 0.75, h * 0.8])?;
        // 4. Construction.
        pg.title(s, base + 3, &num, "Construction")?;
        pg.grid(s, base + 3, asset, "construction", body)?;
        // 5. Clearspace.
        pg.title(s, base + 4, &num, "Zone de protection")?;
        pg.grid(s, base + 4, asset, "clearspace", body)?;
        // 6. A bento board: the logo, the colours, the type.
        pg.title(s, base + 5, &num, "Univers")?;
        let tiles = s.execute(
            "brand.bento",
            &json!({"spread": base + 5, "rect": body, "params": {"style": "centered", "sections": 5, "radius": 10, "gutter": 10, "seed": k}, "brandColors": false}),
        )?;
        let ids: Vec<u64> = tiles["ids"].as_array().map(|a| a.iter().filter_map(Value::as_u64).collect()).unwrap_or_default();
        for (i, tile) in ids.iter().enumerate() {
            if i == 0 {
                s.execute("brand.placeLogo", &json!({"asset": id, "frame": tile}))?;
            } else if let Some((c, _)) = pg.colors.get(i - 1) {
                s.execute("object.fill", &json!({"swatch": c, "ids": [tile]}))?;
            } else {
                s.execute("object.fill", &json!({"swatch": "[Black]", "ids": [tile]}))?;
            }
        }
    }
    // Closing page.
    let last = count - 1;
    pg.fill_rect(s, last, [0.0, 0.0, w, h], &cover)?;
    pg.text(s, last, [M * 2.0, h * 0.4, w - M * 2.0, h * 0.4 + 60.0], "Merci", 48.0, ink)?;
    let end = [studio_name.as_str(), contact.as_str()].iter().filter(|x| !x.is_empty()).copied().collect::<Vec<_>>().join("\n");
    if !end.is_empty() {
        pg.text(s, last, [M * 2.0, h * 0.4 + 66.0, w - M * 2.0, h * 0.4 + 110.0], &end, 14.0, ink)?;
    }
    s.execute("layout.selectPage", &json!({"page": 0})).ok();
    Ok(json!({"pages": count, "concepts": concepts, "warnings": pg.warnings}))
}
