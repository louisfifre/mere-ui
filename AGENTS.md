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
- `crates/nui-ui`: the immediate-mode interface: ids, pointer input, the page's dark and light themes, label, button, toggle, fold, tabs, and the text field and number field (editing model in `edit.rs`, one focused field, keys and clipboard through `nui-base`); hit tests use the last frame's rects. Depends on `nui-base` and `nui-layout` only.
- `demos/shell-boxes`: the world preview's page shell laid out at 1400, 1200 and 1000 px and drawn as named boxes and the gallery of the page's grid rows and absolute boxes (`cargo test -p shell-boxes` writes `target/tmp/shell_<width>.png` and `target/tmp/gallery.png`).
- `demos/relief`: a world preview's relief from a real `out` folder, in a window or as a picture.
- `demos/widgets`: the widgets in the page's looks: a live window, `--shot`, and `cargo test -p widgets` writing `target/tmp/widgets_dark.png` and `widgets_light.png`, plus the controls sheet (folds, strip, choice, details, number fields): the same test run writes `target/tmp/controls_<theme>.png`, and `--controls` opens it live; the fields sheet (text and number fields driven by scripted events): the same test run writes `target/tmp/fields_<theme>.png`, and `--fields` opens it live.
- `docs/design.md`: the design dossier (module tree, reduced flex/grid, text field, build plan, risks).
