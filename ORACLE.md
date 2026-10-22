# Oracle Brand Guide

Fork d'ArtCraft DesignCraft 0.11 (MIT OR Apache-2.0, `NOTICE` conservé), renommé et adapté pour la
suite Oracle : mise en page, pages maîtresses, styles, liens, nuances, PDF avec fonds perdus — et
des **guides de marque liés à un projet Oracle Brand Kit**. Local, sans compte, sans IA.

## Ce qui change par rapport à DesignCraft

- Nom, identifiants, extension `.oracle-guide`, liens GitHub (Marvingxx) ; accent rouge Oracle
  (#d31c3b) ; plus de boutons Discord ni de liens getartcraft.com (`tools/brand_oracle_guide.py`).
- `crates/engine/src/cmd/brand.rs` : les commandes `brand.*`, et le lien `settings.brand` du document.
- `crates/ui-egui/src/panels/brand.rs` : le panneau **Marque** (Fenêtre › Marque, colonne des panneaux).

## Le lien avec le Brand Kit

Le guide suit la marque à travers ce que la mise en page sait déjà faire :

| Donnée du Brand Kit | Dans le guide |
|---|---|
| nom, client, version, règles, interdits | variables de texte `Marque · Nom`, `Marque · Zone de protection`, `Interdit 1`… |
| couleurs | nuances (groupe « Marque ») et variables `Couleur 1 · HEX`, `· RVB`, `· CMJN`, `· Pantone`, `· RAL` |
| typographies | variables `Typo 1 · Famille`, `· Usage`, `· Graisses` |
| logos | images liées dans des cadres étiquetés `oracle:logo:<id>@<rôle>` |
| rendus Mockup Studio | images liées dans des cadres étiquetés `oracle:mockup:<id>@<nom de scène>` |

- `brand.link` relie le document (chemin absolu et relatif au document : le couple guide + marque
  peut être déplacé). Rien n'est modifié tant que le designer ne synchronise pas.
- `brand.status` liste les différences (valeur avant → après) ; `brand.sync` applique celles qui
  sont cochées, en une seule étape d'annulation. Une nuance renommée dans le Brand Kit est renommée
  (ses usages suivent) ; un logo modifié est rechargé, un logo remplacé est relié au nouveau fichier.
- Un texte libre (sans variable) n'est jamais touché ; retirer l'étiquette d'un cadre le déconnecte.
- `brand.placeLogo`, `brand.placeMockup`, `brand.placePalette`, `brand.placeTypes`, `brand.text`
  posent les blocs ; la palette et le bloc typographies se reconstruisent à la demande.
- **Modèles personnels** : un guide fait pour une marque sert de modèle à une autre. Ouvrez-le,
  « Changer… » de projet de marque, enregistrez sous un autre nom, puis mettez à jour : les logos sont
  retrouvés par rôle, les nuances de la palette par position (la 1re couleur prend la case de la
  1re), les variables prennent les nouvelles valeurs ; les textes libres ne bougent pas.
- Une variable ne se coupe jamais en fin de ligne : les valeurs restent courtes (le profil CMJN a
  sa propre variable) et une valeur vide s'affiche « — ».
- CMJN, Pantone et RAL sont ceux saisis dans le Brand Kit (avec leur profil ICC), jamais calculés.
  Les nuances sont en RVB (valeur HEX).
- **PDF/X-4 (impression)** : l'export ajoute à chaque page un espace `DefaultRGB` (profil sRGB) :
  les couleurs vectorielles RVB (logos SVG, nuances RVB) et les images non étiquetées deviennent
  gérées sous l'intention de sortie CMJN ; le gris et le CMJN restent en valeurs d'appareil. Le
  contrôle intégré (`oracle-guide-cli validate-pdf`) passe sur le guide de démo. Aucun contrôle
  externe (preflight d'un imprimeur, Acrobat) n'a été fait.

## Tests

```sh
cargo test -p oracle-guide-engine --lib brand::     # lien, synchronisation, modèle réutilisé, guide 8 pages → PDF
cargo test -p oracle-guide-ui-egui --lib brand_tests # le panneau Marque
python ../tools/demo_brand_guide.py                 # démo de bout en bout (données fictives)
```

Quatre tests d'origine de DesignCraft échouent sous Windows (séparateurs de chemins, privilège de
lien de répertoire, `/dev/zero`) : ils échouent de la même façon sur DesignCraft non modifié.
