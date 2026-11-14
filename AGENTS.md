# rust-native-ui

A small native interface toolkit in Rust, owned end to end and grown piece by piece as applications need it. Its first user is the world preview (the `world-preview` repository), then the Outpost Tools launcher. Windows first, written so that another system only replaces one crate.

## Rules that hold everywhere

- No third-party crate, except Microsoft's `windows` (bindings to the system's own API), and only inside `nui-win32`. When a need is already solved elsewhere (Taffy, microui, egui, Clay), read how they did it and write our own reduced version; if code is ever copied, keep its licence notice next to it.
- Only `nui-win32` names Windows. Every other crate speaks to the system through the traits of `nui-base`, so a Linux backend is one sibling crate.
- Simple over fine-grained: build the subset the applications use, not the general case.
- Every visible step can be checked by an offscreen picture (`--shot`), the same frame the window shows.
- Everything observed at runtime goes through the hub in `nui_base::log`.
- Repository content is English. Conversations with the maintainer are French. Comments say why, not what.
- Private: no remote is published from here.

## Where things live

- `crates/nui-base`: geometry, colours, matrices, the orbit camera, draw lists, the `Gfx` and `App` traits, the log hub. No OS code, no dependency.
- `crates/nui-png`: the PNG writer (deflate with fixed Huffman codes) and a reader for the files it writes, for picture tests.
- `crates/nui-win32`: the Windows backend: Win32 window, Direct3D 11 for meshes, Direct2D and DirectWrite for the interface, offscreen rendering.
- `crates/nui-layout`: pure layout: flex (build step 7), the reduced grid, absolute boxes against their parent and the layout side of overflow (build step 8), tree, measure callbacks, a debug dump. No OS code.
- `crates/nui-ui`: the immediate-mode interface: ids, pointer input, the page's dark and light themes, label, button, toggle, fold, tabs, the text field and number field, the ellipsis text, the vertical scroll area with the page's thin scrollbar, and the list row, legend head and history trend row (editing model in `edit.rs`, one focused field, keys and clipboard through `nui-base`); hit tests use the last frame's rects. Depends on `nui-base` and `nui-layout` only.
- `demos/shell-boxes`: the world preview's page shell laid out at 1400, 1200 and 1000 px and drawn as named boxes and the gallery of the page's grid rows and absolute boxes (`cargo test -p shell-boxes` writes `target/tmp/shell_<width>.png` and `target/tmp/gallery.png`).
- `demos/preview`: the world preview's window: the shell (step 13: toolbar, three columns, the real settings sections and static values) and, with `--data DIR`, the 2D map of an out folder in the stage (step 14: the five views, pan, zoom, grid, cross, ticks, structure marks, probe), in a window or as a picture, and the 3D view inside the stage (step 15: `scene.rs`, shared with `demos/relief`; left-drag turns, the wheel zooms, `--camera`), and the Section view (step 16: `section.rs` the model and the ore play, `cut.rs` the look, the sheet, the strip and the probe; wheel, drag, strip, "Another cut", the ORES list; `--cut`, `--ore`, `--cut-look`) (`cargo test -p preview` writes `target/tmp/preview_<width>.png`, `map_*.png`, `relief3d_*.png`, `section_*.png`; `docs/shots/preview-*.png`, `map-*.png`, `stage-3d-*.png` and `section-*.png` are the pictures). `tools/page_paint.mjs` and `tools/page_section.mjs` are the reference runs of the page's own `paint()` and section code in Node, which `tests/page_paint.rs` and `tests/page_section.rs` compare byte for byte. It grows into the native preview over steps 17 and 18.
- `demos/relief`: the preview's 3D scene alone in a window (or as a picture) with a facts plate; it reads the `out` folder through the `preview` crate.
- `demos/widgets`: the widgets in the page's looks: a live window, `--shot`, and `cargo test -p widgets` writing `target/tmp/widgets_dark.png` and `widgets_light.png`, plus the controls sheet (folds, strip, choice, details, number fields): the same test run writes `target/tmp/controls_<theme>.png`, and `--controls` opens it live; the fields sheet (text and number fields driven by scripted events): the same test run writes `target/tmp/fields_<theme>.png`, and `--fields` opens it live; the lists sheet (biome list with scrollbar, narrow list with absent rows, trend rows): the same test run writes `target/tmp/lists_<theme>.png`, and `--lists` opens it live.
- `docs/design.md`: the design dossier (module tree, reduced flex/grid, text field, build plan, risks).
