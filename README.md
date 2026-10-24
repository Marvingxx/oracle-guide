<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/brand/oracle-logo-white.svg">
    <img alt="Oracle" src="docs/brand/oracle-logo.svg" width="200">
  </picture>
</p>

<p align="center">
  <img src="assets/app-icon/hicolor/256x256/apps/app.oracle.oracle-guide.png" width="128" height="128" alt="Oracle Brand Guide icon">
</p>

<h1 align="center">Oracle Brand Guide</h1>

<p align="center">
  <b>Page layout and publishing; an open-source, modern desktop publishing and page layout studio, built in pure Rust.</b>
</p>

<p align="center">
  A fast, open-source desktop publishing and page layout workflow. It runs natively on macOS,
  Windows and Linux, and in the browser via WebAssembly.<br>
</p>

<p align="center">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-4d7a0a?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on macOS, Windows, Linux and the web" src="https://img.shields.io/badge/runs%20on-macOS%20%C2%B7%20Windows%20%C2%B7%20Linux%20%C2%B7%20Web-7bb51c?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-4d7a0a?style=flat-square">
  <img alt="Agent-drivable over MCP" src="https://img.shields.io/badge/agents-MCP-7bb51c?style=flat-square">
  <br>
</p>

<p align="center">
  <a href="https://github.com/Marvingxx/oracle-guide/releases/latest"><b>Download Oracle Brand Guide</b></a> ·
  <a href="https://github.com/Marvingxx/oracle-suite">Oracle Suite</a>
</p>


<br>

<p align="center">
  <img src="docs/images/ui-spread.png" alt="Oracle Brand Guide with the sample magazine open on its cover, the tools on the left and the Properties panel on the right" width="100%">
  <br><sub><b>Quarterly, Spring Issue</b>: threaded three-column body text, a wrapped pull quote and parent-page folios, all set by Oracle Brand Guide's own paragraph composer.</sub>
</p>


<p align="center">
  <a href="#the-sample-magazine">The sample magazine</a> ·
  <a href="#why-oracle-guide">Why Oracle Brand Guide</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#web">Web</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#downloads">Downloads</a> ·
  <a href="#the-crafting-apps">The Crafting Apps</a> ·
  <a href="#license-and-credits">License and credits</a>
</p>

## The sample magazine

Every page below was laid out by Oracle Brand Guide from code (`crates/engine/src/sample.rs`) and exported
by its own renderer. Try it yourself with `File → New → Sample Document`, or start the app with
`--sample`.

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/page-cover.png" alt="Magazine cover for The Spring Issue, No. 01: a dusk landscape of layered purple hills under a large pale sun, with the white serif headline The Quiet Art of Layout and an italic deck below" width="100%"><p align="center"><sub><b>Cover.</b> A full-bleed graphic frame, display type and an italic deck.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/page-2.png" alt="Feature opener titled Notes on the Grid, with an orange FEATURE · DESIGN kicker over a rule, an italic deck, a wide landscape picture with a small caption, and two columns of justified body text above a QUARTERLY folio" width="100%"><p align="center"><sub><b>Styles.</b> Kicker rule, headline, deck, caption and justified two-column body.</sub></p></td>
</tr>
<tr>
<td width="50%" valign="top"><img src="docs/images/page-3.png" alt="Three-column page of justified, hyphenated body text that wraps around a shaded pull quote reading The best layouts disappear. What remains is the story., with a gradient rule and SPRING ISSUE folio at the foot" width="100%"><p align="center"><sub><b>Threading, columns and text wrap.</b> One story flows through three columns and around the pull quote.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/page-4.png" alt="Coming Next page titled Color, Ink and Paper on a plum background, with four swatch circles labeled Ink Plum, Sunset, C=100 M=0 Y=0 K=0 and Paper Warm, above a short paragraph of body text" width="100%"><p align="center"><sub><b>Swatches.</b> Named colors and a CMYK process swatch, laid out as a palette.</sub></p></td>
</tr>
</table>

## Why Oracle Brand Guide

- **Familiar.** A familiar layout, tools, menus, panels and shortcuts: spreads and parent pages,
  master pages, text frames, threaded text, paragraph and character styles, tables, text wrap and
  more. Everything is right where you expect it.
- **Beautiful type.** A Knuth–Plass paragraph composer (plus single-line), dictionary hyphenation
  (the public-domain Moby word list plus our own trained patterns), word, letter and glyph-scaling
  justification, keeps, optical margin alignment, columns, baseline grid, tabs, rules and shading.
  Line breaks are identical on screen and in PDF.
- **Fast.** Multithreaded SIMD rendering (vello_cpu), copy-on-write documents with O(1) undo
  snapshots, and cached composition.
- **Open.** A documented native format, IDML import and export, INDD import (through
  [indd-utils](https://github.com/Marvingxx/indd-utils)), editable .pptx export, and PNG, PDF
  (PDF/X-4, PDF/A-2b) and EPUB export.

  No subscription, no licence server, no telemetry.
- **Agent-native.** Every menu item, tool gesture, panel control and dialog can be driven over a
  JSON control channel and an **MCP server**, so Claude and other agents can lay out and edit
  documents like a designer. See [`docs/control-protocol.md`](docs/control-protocol.md) and
  [`docs/mcp.md`](docs/mcp.md).
- **Everywhere.** One Rust codebase for desktop and the web.

## Quick start

The egui frontend requires Rust 1.95 or newer (including its shared `craft-ui` components).

```sh
cargo run --release -p oracle-guide                         # desktop app (start screen)
cargo run --release -p oracle-guide -- --sample             # open the sample magazine
cargo run --release -p oracle-guide -- --sample --control 7979   # + JSON control channel
cargo run --release -p oracle-guide-cli -- run --sample --all-pages out/       # headless: render every page to PNG
cargo run --release -p oracle-guide-cli -- commands         # list every command
cargo run --release -p oracle-guide-cli -- script --help    # command script syntax (docs/agents.md)
cargo xtask ci                                             # fmt, clippy, tests, assets, layering, wasm
```

Japanese text (UI and documents) uses fonts from
[storytold/craft-fonts](https://github.com/Marvingxx/craft-fonts), an optional build input (font
files are never committed here; see the public
[craft-fonts integration guide](https://github.com/Marvingxx/craft-fonts/blob/main/docs/integration.md)
for build instructions and craftrules
[`standards/fonts.md`](https://github.com/storytold/craftrules/blob/main/standards/fonts.md)
for the shared standard):

```sh
git clone https://github.com/Marvingxx/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p oracle-guide   # absolute path
```

Without it, Japanese falls back to the system's fonts (none on the web). Release builds always
include it.

To drive a running app, send JSON lines to `127.0.0.1:7979`. The protocol is described in
[`docs/control-protocol.md`](docs/control-protocol.md).

### Logs

The desktop app writes its `log` records to standard error and to `logs/oracle-guide.log` in the
settings directory, beside `ui.json` and `prefs.json` (Linux `~/.config/oracle-guide/logs/`, or
`$XDG_CONFIG_HOME/oracle-guide/logs/`; macOS `~/Library/Application Support/Oracle Brand Guide/logs/`;
Windows `%APPDATA%\Oracle Brand Guide\logs\`). A start launched from a desktop menu or the Dock has no
terminal, so this file is what to attach to a bug report: the graphics adapters found and the
one drawing, why the app stopped (also when its window never opened), the crash guard's panic
report and a failed crash-recovery save land there. Each launch moves the previous log to `oracle-guide.1.log`
(and that one to `oracle-guide.2.log`), so the log of a run that crashed survives the next start.
The file stops growing at 16 MiB. `--version` writes no file.

| Variable | Effect |
|---|---|
| `RUST_LOG` | Log levels for standard error and the log file. Default: `info` for Oracle Brand Guide's own crates, `warn` for everything else. env_logger-style directives replace that, e.g. `RUST_LOG=debug`, `RUST_LOG=warn,oracle_guide_render=trace` or `RUST_LOG=info,wgpu_core=warn`; a directive ending in `*` covers every target starting with it (`oracle-guide*=debug`). |
| `WGPU_BACKEND` | The graphics backend that composites the window: `dx12`, `vulkan`, `gl` or `metal` (a comma-separated list lets wgpu choose among them). Default: DirectX 12 on Windows, Vulkan on Linux, Metal on macOS, with OpenGL as the fallback. Setting it also turns off the start-up fallback described below. |
| `WGPU_POWER_PREF` | The kind of GPU that draws the window: `low` (integrated), `high` (discrete) or `none` (the system's order). Default: the GPU that drives the primary display, then `low` on Windows and macOS and `none` elsewhere. |
| `WGPU_ADAPTER_NAME` | The graphics adapter by (part of) its name, any case (`nvidia`, `radv`, `intel`); the log lists the names. |
| `WGPU_VALIDATION_INDIRECT_CALL` | `1` turns wgpu's check of indirect draw arguments back on. Oracle Brand Guide draws nothing indirectly and turns it off: the compute shader it needs failed to compile on an Intel Mac and lost the graphics device (#334). |

The logger is `apps/oracle-guide/src/logging.rs`; the web build logs to the browser console instead.

### Portable mode

A file named `portable.txt` next to the executable (on macOS, next to `Oracle Brand Guide.app`) makes the install portable: settings, logs and
crash recovery are kept in a `Oracle Brand GuideData` folder beside it instead of the per-user folders
above. The Windows portable zip ships with one; delete it to go back to `%APPDATA%\Oracle Brand Guide`.
If `Oracle Brand GuideData` cannot be written (a copy unzipped into Program Files), the per-user folders
are used instead and the log and the status bar say so.
`ORACLE_GUIDE_CONFIG_DIR` still takes precedence. The code is `apps/oracle-guide/src/profile.rs`.

### Graphics backend and processor

The canvas is rendered on the CPU; the GPU only composites the interface, through wgpu, so any
adapter that can show the window will do. The code is `apps/oracle-guide/src/gpu.rs` (the adapter)
and `apps/oracle-guide/src/gpu/backend.rs` (the backend).

- **Backend.** A graphics driver that faults takes the process down before any Rust code can
  catch it, so each start creates its wgpu instance with one backend, chosen before the window
  exists: on Windows DirectX 12, then OpenGL, then Vulkan, each on its own (creating an OpenGL
  instance crashed inside AMD's `atio6axx.dll`, #167, and creating a Vulkan instance loads every
  installed Vulkan driver, of which a faulty one can crash the app); on Linux Vulkan with OpenGL
  beside it, then OpenGL alone; on macOS Metal. Seen in the wild: on an AMD hybrid-graphics laptop
  (a Radeon RX 6800M beside an integrated Radeon) the AMD Vulkan driver faulted at the first
  present, which is why DirectX 12 is the Windows default.
- **A start that never showed a frame is remembered:** `gpu.json` in the settings directory
  records the backend being tried until the first frame has been presented. A record left behind
  means that backend crashed (or the app was killed) at start-up, so the next start tries the next
  backend and says so in the status bar. The list starts over once every backend has failed, or
  with a new Oracle Brand Guide version; delete the file or set `WGPU_BACKEND` to start afresh.
- **Adapter.** Among the adapters of those backends, the app picks the one that draws the window
  (eframe's `native_adapter_selector`): adapters that report they can't present to the window are
  never tried; then the one `WGPU_ADAPTER_NAME` names; hardware before software (llvmpipe, WARP);
  the native backends before OpenGL; the GPU that drives a display, and among those the one that
  drives the primary display; then the power preference; then a GPU's DirectX 12 adapter before its
  Vulkan one. A GPU without a monitor is the wrong one even when it can present: Windows has to
  copy every frame to the GPU that shows it, and an integrated GPU's driver reset under that,
  losing the graphics device or blanking every monitor for minutes. Windows says which GPU drives
  which display (`EnumDisplayDevices`), Linux through `/sys/class/drm` (a built-in `eDP`, `LVDS` or
  `DSI` panel counts as the primary display); on macOS, or when nothing matches, the power
  preference decides: power saving on Windows and macOS, the system's order elsewhere (Mesa puts
  the GPU the desktop runs on first, and a Wayland compositor may refuse frames from another GPU).
  `WGPU_POWER_PREF` replaces this with the user's choice of kind.
- **If the window fails on its adapter while starting up** (an error from wgpu, such as a lost
  device, or a panic inside wgpu, egui-wgpu or naga during the first frames), the app starts
  again without that adapter and tries the next one, telling which in the status bar; the run that
  failed keeps its log as `oracle-guide.1.log`. When no adapter of the start's backends is left, it
  starts again with the next backend. Software renderers (WARP, llvmpipe) come last: they are
  passed over while another backend is left to try, and allowed once no graphics processor could
  show the window. On Unix the new app replaces the process (an AppImage stays mounted); on
  Windows it starts beside it. Adapters are told apart by backend and PCI ids, or by name where the
  backend reports no ids (Metal lists every GPU of a dual-GPU Mac as `0000:0000`). What was left
  out travels in the restarted app's `--gpu-skip=` argument (`Vulkan:1002:164e`, `Metal:Intel
  Iris Pro Graphics`, or a whole backend such as `Dx12`), one more on each restart, so the
  restarts end. When nothing is left, the app logs why and exits with an error. Documents
  recovered after a crash stay in the recovery folder until they are saved or closed, so a restart
  never loses them.
- **If the window's first frame is late** (20 seconds after the app was created and the files
  named at the start opened), the log and the status bar say so. The adapter counts as hanging,
  and the app starts again without it, only when the UI thread has been stuck in the graphics
  stack, outside Oracle Brand Guide's own code, for 60 seconds while the window wasn't known to be
  minimized, covered or in the background. A large document composing on a slow machine never
  counts.
- **The log** lists the GPUs that drive a display (`display GPUs (PCI vendor:device): 10de:2204
  (primary)`), every adapter with whether it drives a display, the one drawn on and why (`drawing
  on NVIDIA GeForce RTX 3090 (Dx12, DiscreteGpu): it drives the primary display`), and the driver
  version. That is the first thing to look for in a report of a window that stays black or never
  opens.
- On Linux with Mesa, `MESA_VK_DEVICE_SELECT=10de:2f04!` (vendor:device as `vulkaninfo --summary`
  or `lspci -nn` show it) leaves only that GPU to Vulkan, and `DRI_PRIME=1` picks the other GPU.

### Web

```sh
cd apps/oracle-guide-web && trunk build --release          # → dist/web (serve it with any static server)
cd apps/oracle-guide-web && trunk serve --release          # http://127.0.0.1:8767
```

You need [trunk](https://trunkrs.dev) and the `wasm32-unknown-unknown` target. The same app runs
through eframe's web runner on WebGPU, falling back to WebGL2 (`?webgl` forces it; `?sample` opens
the sample magazine). Open and Place use the browser's file picker, and dropping files works too.
Save and Export download the file. The web build has no control channel.

## Architecture

Oracle Brand Guide is an engine-first Cargo workspace with enforced layering (`cargo xtask layers`). The
egui frontend is a separate crate, so the UI can be swapped without touching the engine.

| Layer | Crates |
|---|---|
| L0 | `geom` (paths, units & measurement parsing, corner options) · `color` (CMYK/RGB/Lab, swatches, tints, gradients) |
| L1 | `doc` (spreads, pages, parents, layers, frames, stories, styles) · `fonts` (font DB, shaping, outlines) |
| L2 | `compose` (the text engine) |
| L3 | `render` (vello_cpu) |
| L4 | `tools` (pointer events → commands + overlays) |
| L5 | `engine` (session, history, command registry) |
| L6 | `ui-egui` (publishing UI, control channel) |
| L7 | `apps/oracle-guide`, `apps/oracle-guide-cli`, `apps/oracle-guide-web` |

- **Status and milestones:** [ROADMAP.md](ROADMAP.md) (stage: pre-alpha); feature assessment: [`docs/target-app-parity.md`](docs/target-app-parity.md); known gaps: [`docs/gaps.md`](docs/gaps.md)
- **Contributor and agent rules** (clean-room, asset policy, quality gates): [`AGENTS.md`](AGENTS.md)
- **Bundled assets:** every one is listed with its licence in [`ASSETS.md`](ASSETS.md)
- **App icon and colour:** a calico cat in a polka-dot scarf on Oracle Brand Guide green `#7bb51c`; see [`assets/app-icon/`](assets/app-icon/README.md)

## Downloads

**New to Oracle Brand Guide?** Download it from the [Oracle Brand Guide releases on GitHub](https://github.com/Marvingxx/oracle-guide). That's the easiest way to install it.

**Want a specific build or format?** On GitHub, the [latest release](https://github.com/Marvingxx/oracle-guide/releases/latest) has every build listed below, and [all releases](https://github.com/Marvingxx/oracle-guide/releases) has earlier versions and their notes. `<ver>` in the file names is the version number, and `SHA256SUMS.txt` lists a checksum for every file.

### Windows

| Build | Installer | Portable |
|---|---|---|
| x64 (64-bit Intel/AMD) | `oracle-guide-<ver>-windows-x64.msi` | `oracle-guide-<ver>-windows-x64-portable.zip` |
| arm64 (Snapdragon and other ARM PCs) | `oracle-guide-<ver>-windows-arm64.msi` | `oracle-guide-<ver>-windows-arm64-portable.zip` |
| x86 (32-bit) | `oracle-guide-<ver>-windows-x86.msi` | `oracle-guide-<ver>-windows-x86-portable.zip` |

Installers and executables are code-signed.

### macOS

| Build | File | Notes |
|---|---|---|
| App, universal (Apple silicon + Intel) | `oracle-guide-<ver>-macos-universal.dmg` | Signed and notarized |
| Command-line tool, universal | `oracle-guide-cli-<ver>-macos-universal.zip` | Signed and notarized |

### Linux

| Format | x86_64 | aarch64 (ARM64) | Notes |
|---|---|---|---|
| AppImage | `oracle-guide-<ver>-linux-x86_64.AppImage` | `oracle-guide-<ver>-linux-aarch64.AppImage` | Runs anywhere; updates itself with [AppImageUpdate](https://github.com/AppImageCommunity/AppImageUpdate) (`.zsync` files) |
| Flatpak | `oracle-guide-<ver>-linux-x86_64.flatpak` | `oracle-guide-<ver>-linux-aarch64.flatpak` | Sandboxed; `flatpak install --user <file>` |
| Debian/Ubuntu | `oracle-guide-<ver>-linux-x86_64.deb` | `oracle-guide-<ver>-linux-aarch64.deb` | |
| Fedora/RHEL/openSUSE | `oracle-guide-<ver>-linux-x86_64.rpm` | `oracle-guide-<ver>-linux-aarch64.rpm` | |
| Arch Linux/Manjaro | `oracle-guide-<ver>-linux-x86_64.pkg.tar.zst` | `oracle-guide-<ver>-linux-aarch64.pkg.tar.zst` | `sudo pacman -U <file>`; installs as `app.oracle.oracle-guide`, replacing an AUR package |
| Tarball | `oracle-guide-<ver>-linux-x86_64.tar.gz` | `oracle-guide-<ver>-linux-aarch64.tar.gz` | Unpack anywhere |

RISC-V (riscv64): `oracle-guide-<ver>-linux-riscv64.tar.gz` only, cross-compiled; needs glibc 2.39+ (Ubuntu 24.04 or newer).

#### Community packages

These packages are maintained by community members, not by the Oracle Brand Guide team.

On Gentoo, the [::snakebyte overlay](https://github.com/switch87/snakebyte-overlay) packages the
Linux release as `media-gfx/oracle-guide-bin` (prebuilt) and `media-gfx/oracle-guide` (built from source):

```sh
eselect repository add snakebyte git https://github.com/switch87/snakebyte-overlay.git
emaint sync -r snakebyte
echo 'media-gfx/oracle-guide-bin ~amd64' >> /etc/portage/package.accept_keywords/oracle-guide
emerge --ask media-gfx/oracle-guide-bin
```

### FreeBSD

| Build | File |
|---|---|
| x86_64 | `oracle-guide-<ver>-freebsd-x86_64.tar.gz` |

### Web (WebAssembly)

| Build | File | Notes |
|---|---|---|
| Static site | `oracle-guide-web-<ver>.zip` | Runs in a modern browser; host it on any static server |

## The Oracle suite

Oracle Brand Guide is part of **Oracle**, a personal creative suite, free and local:

| App | What it's for | Code |
|---|---|---|
| **Oracle Suite** | Launcher: installs, updates and opens the apps; the shared asset library | [oracle-suite](https://github.com/Marvingxx/oracle-suite) |
| **Oracle Edit** | Image editing | [oracle-edit](https://github.com/Marvingxx/oracle-edit) |
| **Oracle Vector** | Vector illustration | [oracle-vector](https://github.com/Marvingxx/oracle-vector) |
| **Oracle Brand Kit** · **Oracle Mockup Studio** | Brand identities and client deliveries · 2D mockups | [oracle-brand](https://github.com/Marvingxx/oracle-brand) |
| **Oracle Brand Guide** | Brand guidelines and page layout | [oracle-guide](https://github.com/Marvingxx/oracle-guide) |


## Sources

Oracle Brand Guide is built on [DesignCraft](https://github.com/storytold/designcraft) by storytold (MIT OR
Apache-2.0), taken on 22 October 2026 and developed separately since; its code, rewritten or not, keeps
that licence and the original notices (`NOTICE`, `LICENSE-*`). Every other source (fonts, icons,
libraries, images) is listed with its author and licence in the attribution file.

## License and credits

Oracle Brand Guide is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Required notices are in [NOTICE](NOTICE).

Bundled fonts, icons, images and other assets keep their own open licenses; each one is listed
with its author, source and license in [ASSETS.md](ASSETS.md).

The bundled fonts, and the craft-fonts fonts embedded by release builds, are under the SIL Open
Font License; all UI icons are drawn in code and are original.

The Oracle name and logo in [`docs/brand/`](docs/brand/) are the project's brand marks and are not
covered by this license (see [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt)).

<br>

