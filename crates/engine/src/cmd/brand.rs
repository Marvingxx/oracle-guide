//! Oracle: brand guides linked to a Brand Kit project (`brand.json`).
//!
//! A guide follows the brand through what the layout already has: **text variables** (`Marque ·
//! Nom`, `Couleur 1 · HEX`…), **swatches** in the `Marque` colour group, and **linked graphics** in
//! frames labelled `oracle:logo:<asset id>` or `oracle:mockup:<scene id>`. Nothing is written by
//! the brand on its own: `brand.status` lists what differs, `brand.sync` applies what the designer
//! picks, as one undo step. A frame whose label is removed is disconnected and never touched again;
//! any text around the variables is the designer's and stays as typed.

use std::path::{Path, PathBuf};

use oracle_layout_doc::{BrandLink, Content, ItemId, Selection, vars::VarKind};
use serde_json::{Value, json};

use super::{CommandSpec, bad, cmd, has_doc, str_param};
use crate::{Result, Session};

/// Labels of linked frames.
pub const LOGO_LABEL: &str = "oracle:logo:";
pub const MOCKUP_LABEL: &str = "oracle:mockup:";
/// Label of the palette and type blocks' objects.
pub const PALETTE_LABEL: &str = "oracle:palette";
pub const TYPE_LABEL: &str = "oracle:type";
/// The swatches' colour group.
pub const GROUP: &str = "Marque";
/// The largest `brand.json` or scene file read.
const MAX_JSON: u64 = 16 << 20;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "brand.link",
            "Lier un projet de marque…",
            [],
            None,
            "{path: brand.json or its folder} — the guide follows this Brand Kit project (nothing changes until brand.sync) → {brandName}",
            has_doc,
            link
        ),
        cmd!("brand.unlink", "Délier le projet de marque", [], None, "{} — brand blocks stay as they are, as plain content", has_doc, |s, _| {
            s.edit(|d, _| {
                d.settings.brand = None;
                Ok(json!({}))
            })
        }),
        cmd!(
            query "brand.status",
            "État de la marque",
            [],
            None,
            "{} → {linked, path, brandName, changes: [{key, kind: text|color|logo|mockup|info, label, from, to, apply: bool}], colors, assets, scenes, variables}",
            has_doc,
            status
        ),
        cmd!(
            "brand.sync",
            "Synchroniser avec le Brand Kit",
            [],
            None,
            "{keys?: [change keys from brand.status] (default: all that can apply)} — one undo step → {applied: [keys], warnings}",
            has_doc,
            sync
        ),
        cmd!(
            "brand.placeLogo",
            "Placer un logo de la marque",
            [],
            None,
            "{asset: id, frame?: id (place into) | spread?, rect?: [x0,y0,x1,y1]} — a linked graphic, fitted proportionally, labelled oracle:logo:<id> → {id}",
            has_doc,
            |s, p| place_linked(s, p, "brand.placeLogo")
        ),
        cmd!(
            "brand.placeMockup",
            "Placer un mockup de la marque",
            [],
            None,
            "{scene: id, frame?: id | spread?, rect?} — the scene's render (Mockup Studio), linked, labelled oracle:mockup:<id> → {id}",
            has_doc,
            |s, p| place_linked(s, p, "brand.placeMockup")
        ),
        cmd!(
            "brand.text",
            "Texte avec variables de marque",
            [],
            None,
            "{spread?, rect, text: \"Charte {Marque · Nom}\" — each {Variable name} becomes that text variable, attrs?: type.char attributes for the whole frame} → {id, story}",
            has_doc,
            |s, p| {
                let rect = rect_param(p, "brand.text")?;
                let text = with_variables(s, str_param(p, "text").unwrap_or(""))?;
                let id = new_frame(s, p.get("spread").unwrap_or(&Value::Null), rect, "text", Some(&text))?;
                if let Some(attrs) = p.get("attrs").filter(|a| a.is_object()) {
                    s.edit(|_, sel| {
                        *sel = Selection::items(vec![id]);
                        Ok(())
                    })?;
                    s.execute("type.char", &json!({"attrs": attrs}))?;
                }
                let story = s.active().and_then(|st| st.doc.item(id)).and_then(|it| it.text_frame()).map(|t| t.story.0);
                Ok(json!({"id": id.0, "story": story}))
            }
        ),
        cmd!(
            "brand.placePalette",
            "Placer le bloc palette",
            [],
            None,
            "{spread?, rect: [x0,y0,x1,y1]} — one swatch and its values per brand colour, from the variables and swatches (replaces the spread's previous palette block) → {ids}",
            has_doc,
            place_palette
        ),
        cmd!(
            "brand.placeTypes",
            "Placer le bloc typographies",
            [],
            None,
            "{spread?, rect} — each brand typeface named in itself, with its usage and weights (replaces the spread's previous type block) → {ids, missingFonts}",
            has_doc,
            place_types
        ),
    ]
}

// --- The brand, read from disk ------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Brand {
    pub file: PathBuf,
    pub name: String,
    pub client: String,
    pub version: String,
    pub clear_space: String,
    pub min_size: String,
    pub backgrounds: String,
    pub forbidden: Vec<String>,
    pub colors: Vec<BrandColor>,
    pub fonts: Vec<BrandFont>,
    pub assets: Vec<BrandAsset>,
    pub scenes: Vec<BrandScene>,
}

#[derive(Clone, Debug, Default)]
pub struct BrandColor {
    pub id: String,
    pub name: String,
    pub rgb: [u8; 3],
    pub cmyk: String,
    /// The ICC profile the CMYK values come from.
    pub profile: String,
    pub pantone: String,
    pub ral: String,
}

#[derive(Clone, Debug, Default)]
pub struct BrandFont {
    pub family: String,
    pub usage: String,
    pub weights: String,
}

#[derive(Clone, Debug, Default)]
pub struct BrandAsset {
    pub id: String,
    /// `primaryLogo`, `symbol`…: finds the equivalent master in another brand.
    pub role: String,
    pub label: String,
    pub file: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct BrandScene {
    pub id: String,
    pub name: String,
    /// The last render, when there is one.
    pub render: Option<PathBuf>,
}

fn read_json(path: &Path, what: &str) -> std::result::Result<Value, String> {
    let meta = crate::io::metadata(path).map_err(|e| format!("{what} {} : {e}", path.display()))?;
    if meta.len() > MAX_JSON {
        return Err(format!("{what} {} : fichier trop grand", path.display()));
    }
    let bytes = crate::io::read(path).map_err(|e| format!("{what} {} : {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{what} {} illisible : {e}", path.display()))
}

/// A path stored in a brand project (relative with `/`, or absolute), from the project folder.
fn resolve(dir: &Path, stored: &str) -> PathBuf {
    let p = Path::new(stored);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let mut out = dir.to_path_buf();
    for part in stored.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            x => out.push(x),
        }
    }
    out
}

fn role_fr(role: &str) -> &'static str {
    match role {
        "primaryLogo" => "Logo principal",
        "horizontalLogo" => "Logo horizontal",
        "verticalLogo" => "Logo vertical",
        "symbol" => "Symbole",
        "favicon" => "Favicon",
        "signature" => "Signature",
        "secondaryLogo" => "Version secondaire",
        _ => "Logo",
    }
}

fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let h = hex.trim().strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

/// Reads a Brand Kit project (`brand.json`, and its Mockup Studio scenes).
pub fn load_brand(path: &Path) -> std::result::Result<Brand, String> {
    let file = if crate::io::is_dir(path) { path.join("brand.json") } else { path.to_path_buf() };
    let v = read_json(&file, "projet de marque")?;
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().trim().to_string();
    if v.get("schemaVersion").and_then(Value::as_u64).unwrap_or(0) > 1 {
        return Err("ce projet de marque vient d'une version plus récente d'Oracle".into());
    }
    let name = s(&v, "brandName");
    if name.is_empty() {
        return Err(format!("{} n'est pas un projet de marque (brandName manquant)", file.display()));
    }
    let list = |k: &str| v.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
    let rules = v.get("rules").cloned().unwrap_or(Value::Null);
    let mut colors = Vec::new();
    for c in list("colors") {
        let Some(rgb) = parse_hex(&s(&c, "hex")) else { continue };
        let (cmyk, profile) = match c.get("cmyk").filter(|x| x.is_object()) {
            Some(k) => {
                let n = |key: &str| k.get(key).and_then(Value::as_f64).unwrap_or(0.0);
                (format!("C{} M{} J{} N{}", n("c"), n("m"), n("y"), n("k")), s(k, "profile"))
            }
            None => (String::new(), String::new()),
        };
        colors.push(BrandColor { id: s(&c, "id"), name: s(&c, "name"), rgb, cmyk, profile, pantone: s(&c, "pantone"), ral: s(&c, "ral") });
    }
    let fonts = list("fonts")
        .iter()
        .map(|f| BrandFont {
            family: s(f, "family"),
            usage: s(f, "usage"),
            weights: f.get("weights").and_then(Value::as_array).map(|w| w.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")).unwrap_or_default(),
        })
        .collect();
    let assets = list("assets")
        .iter()
        .map(|a| {
            let label = s(a, "label");
            BrandAsset { id: s(a, "id"), role: s(a, "role"), label: if label.is_empty() { role_fr(&s(a, "role")).into() } else { label }, file: resolve(&dir, &s(a, "source")) }
        })
        .collect();
    let mut scenes = Vec::new();
    for stored in list("mockupScenes").iter().filter_map(Value::as_str) {
        let Ok(sc) = read_json(&resolve(&dir, stored), "scène") else { continue };
        let render = sc.get("lastRender").and_then(|r| r.get("file")).and_then(Value::as_str).map(|f| resolve(&dir, f));
        scenes.push(BrandScene { id: s(&sc, "id"), name: s(&sc, "name"), render });
    }
    Ok(Brand {
        file,
        name,
        client: s(&v, "client"),
        version: s(&v, "version"),
        clear_space: s(&rules, "clearSpace"),
        min_size: s(&rules, "minSize"),
        backgrounds: s(&rules, "allowedBackgrounds"),
        forbidden: rules.get("forbidden").and_then(Value::as_array).map(|f| f.iter().map(|x| s(x, "text")).collect()).unwrap_or_default(),
        colors,
        fonts,
        assets,
        scenes,
    })
}

/// The text variables the brand fills, in order.
pub fn brand_variables(b: &Brand) -> Vec<(String, String)> {
    let mut v = vec![
        ("Marque · Nom".to_string(), b.name.clone()),
        ("Marque · Client".into(), b.client.clone()),
        ("Marque · Version".into(), b.version.clone()),
        ("Marque · Zone de protection".into(), b.clear_space.clone()),
        ("Marque · Taille minimale".into(), b.min_size.clone()),
        ("Marque · Fonds autorisés".into(), b.backgrounds.clone()),
    ];
    for (i, c) in b.colors.iter().enumerate() {
        let n = i + 1;
        let [r, g, bl] = c.rgb;
        v.push((format!("Couleur {n} · Nom"), c.name.clone()));
        v.push((format!("Couleur {n} · HEX"), format!("#{r:02X}{g:02X}{bl:02X}")));
        v.push((format!("Couleur {n} · RVB"), format!("R {r} · V {g} · B {bl}")));
        // A variable never breaks across lines: values stay short, and an empty one shows a dash.
        let or_dash = |v: &str| if v.is_empty() { "—".to_string() } else { v.to_string() };
        v.push((format!("Couleur {n} · CMJN"), or_dash(&c.cmyk)));
        v.push((format!("Couleur {n} · Profil CMJN"), or_dash(&c.profile)));
        v.push((format!("Couleur {n} · Pantone"), or_dash(&c.pantone)));
        v.push((format!("Couleur {n} · RAL"), or_dash(&c.ral)));
    }
    for (i, f) in b.fonts.iter().enumerate() {
        let n = i + 1;
        v.push((format!("Typo {n} · Famille"), f.family.clone()));
        v.push((format!("Typo {n} · Usage"), f.usage.clone()));
        v.push((format!("Typo {n} · Graisses"), f.weights.clone()));
    }
    for (i, t) in b.forbidden.iter().enumerate() {
        v.push((format!("Interdit {}", i + 1), t.clone()));
    }
    v
}

/// Whether a variable name is one the brand fills (so a colour no longer in the brand empties it).
fn is_brand_variable(name: &str) -> bool {
    name.starts_with("Marque · ")
        || ["Couleur ", "Typo "].iter().any(|p| name.strip_prefix(p).is_some_and(|r| r.split(" · ").next().is_some_and(|n| n.parse::<usize>().is_ok())))
        || name.strip_prefix("Interdit ").is_some_and(|n| n.parse::<usize>().is_ok())
}

/// The linked brand, from the document's link (its absolute path, else relative to the
/// document's folder).
pub(super) fn linked_brand(s: &Session) -> std::result::Result<Option<(Brand, BrandLink)>, String> {
    let st = s.active().ok_or("aucun document ouvert")?;
    let Some(link) = st.doc.settings.brand.clone() else { return Ok(None) };
    let abs = PathBuf::from(&link.path);
    let rel = st.path.as_deref().and_then(|p| Path::new(p).parent()).filter(|_| !link.relative.is_empty()).map(|d| resolve(d, &link.relative));
    let file = if crate::io::is_file(&abs) { abs } else { rel.filter(|r| crate::io::is_file(r)).unwrap_or(abs) };
    load_brand(&file).map(|b| Some((b, link)))
}

fn relative_to(from_dir: &Path, to: &Path) -> String {
    let (Ok(a), Ok(b)) = (std::path::absolute(from_dir), std::path::absolute(to)) else { return String::new() };
    let ac: Vec<_> = a.components().collect();
    let bc: Vec<_> = b.components().collect();
    let common = ac.iter().zip(&bc).take_while(|(x, y)| x == y).count();
    if common == 0 {
        return String::new();
    }
    let mut parts: Vec<String> = std::iter::repeat_n("..".to_string(), ac.len() - common).collect();
    parts.extend(bc[common..].iter().map(|c| c.as_os_str().to_string_lossy().to_string()));
    parts.join("/")
}

fn link(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").ok_or_else(|| bad("brand.link", "missing path"))?;
    let brand = load_brand(Path::new(path)).map_err(|e| bad("brand.link", e))?;
    let abs = std::path::absolute(&brand.file).unwrap_or_else(|_| brand.file.clone());
    let doc_dir = s.doc()?.path.as_deref().and_then(|p| Path::new(p).parent().map(Path::to_path_buf));
    let relative = doc_dir.map(|d| relative_to(&d, &abs)).unwrap_or_default();
    let name = brand.name.clone();
    s.edit(|d, _| {
        let swatches = d.settings.brand.as_ref().map(|b| b.swatches.clone()).unwrap_or_default();
        let palette = d.settings.brand.as_ref().map(|b| b.palette.clone()).unwrap_or_default();
        d.settings.brand = Some(BrandLink { path: abs.display().to_string(), relative, swatches, palette });
        Ok(json!({"brandName": name}))
    })
}

// --- What differs ------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Change {
    key: String,
    kind: &'static str,
    label: String,
    from: String,
    to: String,
    apply: bool,
    /// The frame's new label, when it now shows another brand's equivalent.
    relabel: Option<String>,
}

/// The swatch showing brand colour `i`: its own, else (a guide reused for another brand) the
/// swatch at the same position in the palette, when no other colour of this brand holds it.
pub(super) fn swatch_for(link: &BrandLink, d: &oracle_layout_doc::Document, brand: &Brand, i: usize) -> Option<String> {
    let c = brand.colors.get(i)?;
    let exists = |n: &String| d.swatches.iter().any(|sw| &sw.name == n);
    if let Some(n) = link.swatches.get(&c.id).filter(|n| exists(n)) {
        return Some(n.clone());
    }
    let claimed = |n: &String| brand.colors.iter().any(|o| o.id != c.id && link.swatches.get(&o.id) == Some(n));
    link.palette.get(i).filter(|n| exists(n) && !claimed(n)).cloned()
}

/// Frames labelled for the brand: (frame, kind, id, linked asset, its link path).
fn labelled_frames(s: &Session) -> Vec<(ItemId, &'static str, String, Option<oracle_layout_doc::AssetId>, Option<String>)> {
    let Some(st) = s.active() else { return Vec::new() };
    let d = &st.doc;
    let mut out = Vec::new();
    for sp in d.spreads.iter().chain(d.parents.iter()) {
        for it in &sp.items {
            it.walk(&mut |x| {
                let (kind, id) = if let Some(id) = x.label.strip_prefix(LOGO_LABEL) {
                    ("logo", id)
                } else if let Some(id) = x.label.strip_prefix(MOCKUP_LABEL) {
                    ("mockup", id)
                } else {
                    return;
                };
                let asset = match &x.content {
                    Content::Graphic(g) => Some(g.asset),
                    _ => None,
                };
                let path = asset.and_then(|a| d.assets.get(&a)).and_then(|a| a.link.clone());
                out.push((x.id, kind, id.to_string(), asset, path));
            });
        }
    }
    out
}

fn same_file(a: &str, b: &Path) -> bool {
    let a = Path::new(a);
    match (crate::io::canonicalize(a), crate::io::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => std::path::absolute(a).ok() == std::path::absolute(b).ok(),
    }
}

fn changes(s: &mut Session, brand: &Brand, link: &BrandLink) -> Result<Vec<Change>> {
    let mut out = Vec::new();
    let d = s.doc()?.doc.clone();
    // Text variables.
    let wanted = brand_variables(brand);
    for (name, value) in &wanted {
        let current = d.variable(name).and_then(|i| d.text_variables.get(i)).map(|v| match &v.kind {
            VarKind::Custom { text } => text.clone(),
            _ => String::new(),
        });
        if current.as_deref() != Some(value.as_str()) {
            out.push(Change {
                key: format!("text:{name}"),
                kind: "text",
                label: name.clone(),
                from: current.unwrap_or_else(|| "(absente)".into()),
                to: value.clone(),
                apply: true,
                relabel: None,
            });
        }
    }
    for v in &d.text_variables {
        if is_brand_variable(&v.name) && !wanted.iter().any(|(n, _)| *n == v.name) && matches!(&v.kind, VarKind::Custom { text } if !text.is_empty()) {
            let from = match &v.kind {
                VarKind::Custom { text } => text.clone(),
                _ => String::new(),
            };
            out.push(Change { key: format!("text:{}", v.name), kind: "text", label: format!("{} (retirée de la marque)", v.name), from, to: String::new(), apply: true, relabel: None });
        }
    }
    // Swatches.
    for (i, c) in brand.colors.iter().enumerate() {
        let hex = format!("#{:02x}{:02x}{:02x}", c.rgb[0], c.rgb[1], c.rgb[2]);
        let mapped = swatch_for(link, &d, brand, i);
        let mapped = mapped.as_ref();
        let current = mapped.or(Some(&c.name)).and_then(|n| d.swatches.iter().find(|sw| &sw.name == n));
        let current_hex = current.and_then(|sw| match &sw.value {
            oracle_layout_color::SwatchValue::Color { color, .. } => Some(color.to_hex()),
            _ => None,
        });
        let renamed = mapped.is_some_and(|n| *n != c.name);
        if current.is_none() || current_hex.as_deref() != Some(hex.as_str()) || renamed || mapped.is_none() {
            let from = match (current, &current_hex) {
                (Some(sw), Some(h)) => format!("{} {}", sw.name, h.to_uppercase()),
                (Some(sw), None) => sw.name.clone(),
                _ => "(absente)".into(),
            };
            out.push(Change { key: format!("color:{}", c.id), kind: "color", label: format!("Nuance « {} »", c.name), from, to: format!("{} {}", c.name, hex.to_uppercase()), apply: true, relabel: None });
        }
    }
    // Linked frames.
    let statuses: Vec<Value> = s.execute("links.list", &json!({})).ok().and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let status_of = |aid: oracle_layout_doc::AssetId| statuses.iter().find(|r| r["asset"].as_u64() == Some(aid.0)).and_then(|r| r["status"].as_str().map(str::to_string));
    for (frame, kind, tag, asset, path) in labelled_frames(s) {
        // `<id>@<role>` (logos) or `<id>@<scene name>` (mockups); older labels have no `@`.
        let (id, alt) = tag.rsplit_once('@').map(|(i, r)| (i.to_string(), r.to_string())).unwrap_or((tag.clone(), String::new()));
        let mut relabel = None;
        let (name, want) = match kind {
            "logo" => match brand.assets.iter().find(|a| a.id == id).or_else(|| brand.assets.iter().find(|a| !alt.is_empty() && a.role == alt)) {
                Some(a) => {
                    if a.id != id {
                        relabel = Some(format!("{LOGO_LABEL}{}@{}", a.id, a.role));
                    }
                    (a.label.clone(), Some(a.file.clone()))
                }
                None => {
                    out.push(Change { key: format!("info:{}", frame.0), kind: "info", label: format!("Cadre {} : le logo {id} n'est plus dans le Brand Kit (cadre conservé)", frame.0), from: String::new(), to: String::new(), apply: false, relabel: None });
                    continue;
                }
            },
            _ => match brand.scenes.iter().find(|sc| sc.id == id).or_else(|| brand.scenes.iter().find(|sc| !alt.is_empty() && sc.name == alt)) {
                Some(sc) => {
                    if sc.id != id {
                        relabel = Some(format!("{MOCKUP_LABEL}{}@{}", sc.id, sc.name));
                    }
                    (format!("Mockup « {} »", sc.name), sc.render.clone())
                }
                None => {
                    out.push(Change { key: format!("info:{}", frame.0), kind: "info", label: format!("Cadre {} : la scène {id} n'est plus dans le projet (cadre conservé)", frame.0), from: String::new(), to: String::new(), apply: false, relabel: None });
                    continue;
                }
            },
        };
        let Some(want) = want.filter(|w| crate::io::is_file(w)) else {
            out.push(Change { key: format!("info:{}", frame.0), kind: "info", label: format!("{name} : fichier introuvable (jamais rendu ou déplacé)"), from: String::new(), to: String::new(), apply: false, relabel: None });
            continue;
        };
        let key = format!("{kind}:{}", frame.0);
        let to = want.display().to_string();
        let change = |label: String, from: String| Change { key: key.clone(), kind, label, from, to: to.clone(), apply: true, relabel: relabel.clone() };
        match (asset, path) {
            (None, _) => out.push(change(format!("{name} : à placer"), "(vide)".into())),
            (Some(_), Some(p)) if !same_file(&p, &want) => out.push(change(format!("{name} : autre fichier dans le Brand Kit"), p)),
            (Some(a), Some(p)) if status_of(a).as_deref() == Some("modified") => out.push(change(format!("{name} : fichier modifié"), p)),
            (Some(_), None) => out.push(change(format!("{name} : image incorporée, à relier"), "(incorporée)".into())),
            _ if relabel.is_some() => out.push(change(format!("{name} : même fichier, autre identifiant"), to.clone())),
            _ => {}
        }
    }
    Ok(out)
}

fn status(s: &mut Session, _: &Value) -> Result<Value> {
    let found = linked_brand(s).map_err(|e| bad("brand.status", e))?;
    let Some((brand, link)) = found else { return Ok(json!({"linked": false, "changes": []})) };
    let list = changes(s, &brand, &link)?;
    Ok(json!({
        "linked": true,
        "path": brand.file.display().to_string(),
        "brandName": brand.name,
        "changes": list.iter().map(|c| json!({"key": c.key, "kind": c.kind, "label": c.label, "from": c.from, "to": c.to, "apply": c.apply})).collect::<Vec<_>>(),
        "colors": brand.colors.iter().map(|c| json!({"id": c.id, "name": c.name, "hex": format!("#{:02X}{:02X}{:02X}", c.rgb[0], c.rgb[1], c.rgb[2])})).collect::<Vec<_>>(),
        "fonts": brand.fonts.iter().map(|f| json!({"family": f.family, "usage": f.usage})).collect::<Vec<_>>(),
        "assets": brand.assets.iter().map(|a| json!({"id": a.id, "label": a.label, "path": a.file.display().to_string(), "exists": crate::io::is_file(&a.file)})).collect::<Vec<_>>(),
        "scenes": brand.scenes.iter().map(|sc| json!({"id": sc.id, "name": sc.name, "render": sc.render.as_ref().map(|r| r.display().to_string()), "exists": sc.render.as_ref().is_some_and(crate::io::is_file)})).collect::<Vec<_>>(),
        "variables": brand_variables(&brand).into_iter().map(|(n, v)| json!({"name": n, "value": v})).collect::<Vec<_>>(),
    }))
}

// --- Applying ------------------------------------------------------------------------------------

fn sync(s: &mut Session, p: &Value) -> Result<Value> {
    let found = linked_brand(s).map_err(|e| bad("brand.sync", e))?;
    let Some((brand, link)) = found else { return Err(bad("brand.sync", "aucun projet de marque lié (brand.link)")) };
    let only: Option<Vec<String>> = p.get("keys").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect());
    let list = changes(s, &brand, &link)?;
    let picked: Vec<Change> = list.into_iter().filter(|c| c.apply && only.as_ref().is_none_or(|o| o.contains(&c.key))).collect();
    let mut applied = Vec::new();
    let mut warnings = Vec::new();
    let mut new_swatches = Vec::new();
    for c in &picked {
        let r = match c.kind {
            "text" => {
                let name = c.key.trim_start_matches("text:");
                s.execute("variables.define", &json!({"name": name, "type": "custom", "text": c.to})).map(|_| ())
            }
            "color" => {
                let id = c.key.trim_start_matches("color:");
                let Some(index) = brand.colors.iter().position(|x| x.id == id) else { continue };
                let bc = &brand.colors[index];
                let hex = format!("#{:02x}{:02x}{:02x}", bc.rgb[0], bc.rgb[1], bc.rgb[2]);
                let d = &s.doc()?.doc;
                let mapped = swatch_for(&link, d, &brand, index);
                let existing = mapped.clone().or_else(|| d.swatches.iter().find(|sw| sw.name == bc.name).map(|sw| sw.name.clone()));
                let r = match existing {
                    Some(n) => {
                        let mut params = json!({"name": n, "color": hex});
                        if n != bc.name {
                            params["to"] = json!(bc.name);
                        }
                        s.execute("swatch.options", &params).map(|_| ())
                    }
                    None => s.execute("swatch.create", &json!({"name": bc.name, "color": hex})).map(|_| ()),
                };
                if r.is_ok() {
                    new_swatches.push((id.to_string(), bc.name.clone()));
                }
                r
            }
            "logo" | "mockup" => {
                let frame: u64 = c.key.split(':').nth(1).and_then(|x| x.parse().ok()).unwrap_or(0);
                place_into(s, ItemId(frame), Path::new(&c.to)).and_then(|()| match &c.relabel {
                    Some(l) => label(s, &[ItemId(frame)], l),
                    None => Ok(()),
                })
            }
            _ => continue,
        };
        match r {
            Ok(()) => applied.push(c.key.clone()),
            Err(e) => warnings.push(format!("{} : {e}", c.label)),
        }
    }
    // Remember which swatch shows each brand colour, and keep them in the `Marque` group.
    if !new_swatches.is_empty() {
        let names: Vec<String> = new_swatches.iter().map(|(_, n)| n.clone()).collect();
        let has_group = s.doc()?.doc.color_groups.iter().any(|g| g.name == GROUP);
        let grouped = if has_group {
            s.execute("swatch.moveToGroup", &json!({"swatches": names, "group": GROUP}))
        } else {
            s.execute("swatch.newColorGroup", &json!({"name": GROUP, "swatches": names}))
        };
        if let Err(e) = grouped {
            warnings.push(format!("groupe de nuances « {GROUP} » : {e}"));
        }
        s.edit(|d, _| {
            if let Some(l) = d.settings.brand.as_mut() {
                for (id, name) in &new_swatches {
                    l.swatches.insert(id.clone(), name.clone());
                }
                l.palette = brand.colors.iter().map(|c| l.swatches.get(&c.id).cloned().unwrap_or_else(|| c.name.clone())).collect();
            }
            Ok(())
        })?;
    }
    // The relative path follows the document's current folder.
    let doc_dir = s.doc()?.path.as_deref().and_then(|p| Path::new(p).parent().map(Path::to_path_buf));
    if let Some(dir) = doc_dir {
        let rel = relative_to(&dir, &brand.file);
        if s.doc()?.doc.settings.brand.as_ref().is_some_and(|l| l.relative != rel) {
            s.edit(|d, _| {
                if let Some(l) = d.settings.brand.as_mut() {
                    l.relative = rel;
                }
                Ok(())
            })?;
        }
    }
    Ok(json!({"applied": applied, "warnings": warnings}))
}

/// Shows `file` in `frame`: relinks the frame's graphic (all its uses), reloads it when only
/// modified, or places it when the frame is empty; then fits it proportionally.
fn place_into(s: &mut Session, frame: ItemId, file: &Path) -> Result<()> {
    let d = &s.doc()?.doc;
    let it = d.item(frame).ok_or_else(|| bad("brand.sync", format!("cadre {} introuvable", frame.0)))?;
    let current = match &it.content {
        Content::Graphic(g) => d.assets.get(&g.asset).map(|a| (g.asset, a.link.clone())),
        _ => None,
    };
    let path = file.display().to_string();
    match current {
        Some((aid, Some(link))) if same_file(&link, file) => {
            s.execute("links.update", &json!({"asset": aid.0}))?;
        }
        Some((aid, Some(_))) => {
            // Other frames may show the same graphic: only this frame changes file.
            let shared = labelled_or_not_uses(s, aid) > 1;
            if shared {
                s.execute("file.place", &json!({"path": path, "frame": frame.0, "link": true}))?;
            } else {
                s.execute("links.relink", &json!({"asset": aid.0, "path": path}))?;
            }
        }
        _ => {
            s.execute("file.place", &json!({"path": path, "frame": frame.0, "link": true}))?;
        }
    }
    s.execute("object.fittingOptions", &json!({"autoFit": true, "fitting": "fitProportionally", "align": 4, "ids": [frame.0]}))?;
    s.execute("object.fit", &json!({"mode": "fitProportionally", "ids": [frame.0]}))?;
    Ok(())
}

/// How many frames show asset `aid`.
fn labelled_or_not_uses(s: &Session, aid: oracle_layout_doc::AssetId) -> usize {
    let Some(st) = s.active() else { return 0 };
    let mut n = 0;
    for sp in st.doc.spreads.iter().chain(st.doc.parents.iter()) {
        for it in &sp.items {
            it.walk(&mut |x| {
                if matches!(&x.content, Content::Graphic(g) if g.asset == aid) {
                    n += 1;
                }
            });
        }
    }
    n
}

// --- Placing brand blocks ------------------------------------------------------------------------

pub(super) fn rect_param(p: &Value, cmd: &str) -> Result<[f64; 4]> {
    let r = p.get("rect").and_then(Value::as_array).ok_or_else(|| bad(cmd, "missing rect"))?;
    let v: Vec<f64> = r.iter().filter_map(Value::as_f64).filter(|x| x.is_finite()).collect();
    match v[..] {
        [x0, y0, x1, y1] if x1 > x0 && y1 > y0 => Ok([x0, y0, x1, y1]),
        _ => Err(bad(cmd, "rect is [x0, y0, x1, y1] with x1 > x0 and y1 > y0")),
    }
}

pub(super) fn new_frame(s: &mut Session, spread: &Value, rect: [f64; 4], content: &str, text: Option<&str>) -> Result<ItemId> {
    let mut params = json!({"rect": rect, "content": content});
    if !spread.is_null() {
        params["spread"] = spread.clone();
    }
    if let Some(t) = text {
        params["text"] = json!(t);
    }
    let r = s.execute("frame.create", &params)?;
    r["id"].as_u64().map(ItemId).ok_or_else(|| bad("frame.create", "no frame id"))
}

fn label(s: &mut Session, ids: &[ItemId], text: &str) -> Result<()> {
    s.execute("object.label", &json!({"label": text, "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>()}))?;
    Ok(())
}

fn place_linked(s: &mut Session, p: &Value, cmd: &'static str) -> Result<Value> {
    let found = linked_brand(s).map_err(|e| bad(cmd, e))?;
    let Some((brand, _)) = found else { return Err(bad(cmd, "aucun projet de marque lié (brand.link)")) };
    let (prefix, id, file) = if cmd == "brand.placeLogo" {
        let id = str_param(p, "asset").ok_or_else(|| bad(cmd, "missing asset"))?;
        let a = brand.assets.iter().find(|a| a.id == id).ok_or_else(|| bad(cmd, format!("pas de logo {id} dans le Brand Kit")))?;
        (LOGO_LABEL, format!("{id}@{}", a.role), a.file.clone())
    } else {
        let id = str_param(p, "scene").ok_or_else(|| bad(cmd, "missing scene"))?;
        let sc = brand.scenes.iter().find(|x| x.id == id).ok_or_else(|| bad(cmd, format!("pas de scène {id} dans le projet")))?;
        let r = sc.render.clone().ok_or_else(|| bad(cmd, format!("la scène « {} » n'a jamais été rendue dans Mockup Studio", sc.name)))?;
        (MOCKUP_LABEL, format!("{id}@{}", sc.name), r)
    };
    if !crate::io::is_file(&file) {
        return Err(bad(cmd, format!("{} est introuvable", file.display())));
    }
    let frame = match p.get("frame").and_then(Value::as_u64) {
        Some(f) => ItemId(f),
        None => {
            let rect = rect_param(p, cmd)?;
            new_frame(s, p.get("spread").unwrap_or(&Value::Null), rect, "graphic", None)?
        }
    };
    label(s, &[frame], &format!("{prefix}{id}"))?;
    s.execute("file.place", &json!({"path": file.display().to_string(), "frame": frame.0, "link": true}))?;
    s.execute("object.fittingOptions", &json!({"autoFit": true, "fitting": "fitProportionally", "align": 4, "ids": [frame.0]}))?;
    s.execute("object.fit", &json!({"mode": "fitProportionally", "ids": [frame.0]}))?;
    Ok(json!({"id": frame.0}))
}

/// Removes a spread's objects carrying `label` (a block placed before), and returns their
/// spread index.
fn remove_labelled(s: &mut Session, spread: &Value, text: &str) -> Result<()> {
    let index = spread.as_u64().unwrap_or(0) as usize;
    let d = &s.doc()?.doc;
    let Some(sp) = d.spreads.get(index) else { return Ok(()) };
    let ids: Vec<u64> = sp.items.iter().filter(|it| it.label == text).map(|it| it.id.0).collect();
    if !ids.is_empty() {
        s.execute("edit.clear", &json!({"ids": ids}))?;
    }
    Ok(())
}

/// Text with variable instances: `{Name}` becomes the variable's character.
fn with_variables(s: &Session, template: &str) -> Result<String> {
    let d = &s.doc()?.doc;
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let name = &after[..end];
        match d.variable(name).and_then(oracle_layout_doc::vars::var_char) {
            Some(c) => out.push(c),
            None => return Err(bad("brand", format!("variable « {name} » absente : synchronisez d'abord"))),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn place_palette(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "brand.placePalette";
    let found = linked_brand(s).map_err(|e| bad(cmd, e))?;
    let Some((brand, link)) = found else { return Err(bad(cmd, "aucun projet de marque lié (brand.link)")) };
    if brand.colors.is_empty() {
        return Err(bad(cmd, "le Brand Kit n'a pas de couleur"));
    }
    let [x0, y0, x1, y1] = rect_param(p, cmd)?;
    let spread = p.get("spread").cloned().unwrap_or(Value::Null);
    remove_labelled(s, &spread, PALETTE_LABEL)?;
    let n = brand.colors.len();
    let cols = n.min(4);
    let rows = n.div_ceil(cols);
    let gap = 12.0;
    let w = (x1 - x0 - gap * (cols as f64 - 1.0)) / cols as f64;
    let h = (y1 - y0 - gap * (rows as f64 - 1.0)) / rows as f64;
    let mut ids = Vec::new();
    for (i, c) in brand.colors.iter().enumerate() {
        let (cx, cy) = (x0 + (i % cols) as f64 * (w + gap), y0 + (i / cols) as f64 * (h + gap));
        let swatch_h = h * 0.55;
        let swatch = swatch_for(&link, &s.doc()?.doc, &brand, i).unwrap_or_else(|| c.name.clone());
        let block = new_frame(s, &spread, [cx, cy, cx + w, cy + swatch_h], "unassigned", None)?;
        s.execute("object.fill", &json!({"swatch": swatch, "ids": [block.0]})).map_err(|e| bad(cmd, format!("nuance « {swatch} » : {e} — synchronisez d'abord")))?;
        let k = i + 1;
        let text = with_variables(s, &format!("{{Couleur {k} · Nom}}\nHEX {{Couleur {k} · HEX}}\n{{Couleur {k} · RVB}}\nCMJN {{Couleur {k} · CMJN}}\n{{Couleur {k} · Profil CMJN}}\nPantone {{Couleur {k} · Pantone}}"))?;
        let t = new_frame(s, &spread, [cx, cy + swatch_h + 6.0, cx + w, cy + h], "text", Some(&text))?;
        ids.extend([block, t]);
    }
    label(s, &ids, PALETTE_LABEL)?;
    s.edit(|_, sel| {
        *sel = Selection::items(ids.clone());
        Ok(())
    })?;
    Ok(json!({"ids": ids.iter().map(|i| i.0).collect::<Vec<_>>()}))
}

fn place_types(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "brand.placeTypes";
    let found = linked_brand(s).map_err(|e| bad(cmd, e))?;
    let Some((brand, _)) = found else { return Err(bad(cmd, "aucun projet de marque lié (brand.link)")) };
    if brand.fonts.is_empty() {
        return Err(bad(cmd, "le Brand Kit n'a pas de typographie"));
    }
    let [x0, y0, x1, y1] = rect_param(p, cmd)?;
    let spread = p.get("spread").cloned().unwrap_or(Value::Null);
    remove_labelled(s, &spread, TYPE_LABEL)?;
    let n = brand.fonts.len();
    let gap = 14.0;
    let h = (y1 - y0 - gap * (n as f64 - 1.0)) / n as f64;
    let mut ids = Vec::new();
    let mut missing = Vec::new();
    for (i, f) in brand.fonts.iter().enumerate() {
        let k = i + 1;
        let y = y0 + i as f64 * (h + gap);
        // The family's name, set in that family when it is installed.
        let sample = with_variables(s, &format!("{{Typo {k} · Famille}}"))?;
        let title = new_frame(s, &spread, [x0, y, x1, y + h * 0.5], "text", Some(&sample))?;
        s.edit(|_, sel| {
            *sel = Selection::items(vec![title]);
            Ok(())
        })?;
        let applied = s.execute("type.char", &json!({"attrs": {"fontFamily": f.family, "size": (h * 0.32).clamp(10.0, 48.0)}}));
        let font_ok = applied.is_ok();
        if !font_ok || !font_installed(s, &f.family) {
            missing.push(f.family.clone());
        }
        let info = with_variables(s, &format!("{{Typo {k} · Usage}} — {{Typo {k} · Graisses}}"))?;
        let t = new_frame(s, &spread, [x0, y + h * 0.5 + 4.0, x1, y + h], "text", Some(&info))?;
        ids.extend([title, t]);
    }
    label(s, &ids, TYPE_LABEL)?;
    s.edit(|_, sel| {
        *sel = Selection::items(ids.clone());
        Ok(())
    })?;
    Ok(json!({"ids": ids.iter().map(|i| i.0).collect::<Vec<_>>(), "missingFonts": missing}))
}

/// Whether a font family the document uses is available (bundled, installed or document fonts).
pub(super) fn font_installed(s: &mut Session, family: &str) -> bool {
    let list = s.execute("font.list", &json!({})).ok();
    !list.as_ref().and_then(Value::as_array).is_some_and(|l| {
        l.iter().any(|f| f["family"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(family)) && f["missing"].as_bool() == Some(true))
    })
}

#[cfg(test)]
mod tests;
