# Handoff: steps 0 to 6, to run as a workflow

Written 2026-11-14 at the end of the session that designed the kit. The next session starts here and launches
a Workflow (the maintainer asked for one in words: "le mieux, ce serait de lancer un workflow", so the opt-in
is explicit). Read `AGENTS.md` and `docs/design.md` first; this file says where the work stands and how to
finish steps 0 to 6.

## Status: steps 0 to 13 done (2026-11-14)

The build run finished. `relief <data-dir>` draws `C:\Games\preview\out` in 3D in a window and as `--shot`; the
shots in `docs/shots` were compared with a capture of the page's 3D view (`world-preview/scripts/capture.ps1`,
`/?view=height,relief3d`) and match in framing, colours, water and edges. A review pass then added 4x MSAA
(the page asks WebGL for antialias), hairlines of one logical pixel, the page's biome total and wording,
OrbitControls speeds, a camera kept above the ground within a zoom range, and robustness fixes in the window
(no `&mut` aliasing across re-entrant calls, a cleanup guard, safe capture, device-loss rebuild).

Not exercised at runtime: device loss, capture lost mid-drag, a `WM_DPICHANGED` between monitors, and a
look at the live window's pixels (only its start, answer and clean exit were checked).

Later the same day: text is in IBM Plex Mono (`fonts/`, loaded by `nui-win32/src/fonts.rs` through
`IDWriteFactory5::UnpackFontFile`; the Pi cut's arrows and signs are reached by DirectWrite's coverage
matching inside the one family, seen in `tests/fonts.rs`'s picture, not a documented guarantee; the 600 face
is loaded but unused). `fonts/OFL.txt` was written from memory, as no copy was on the machine: compare it
with IBM's official LICENSE.txt before anything is published. Build plan step 7 is done: `nui-layout` (flex,
28 rect tests, dump) and `demos/shell-boxes`, whose pictures at 1400, 1200 and 1000 px match the page's
column widths; heights and the strip width the page does not fix are marked PLACEHOLDER there.

Build plan step 8 is done: `nui-layout` has a reduced grid (`grid.rs`: columns of a length, `auto` or
`minmax(0, Nfr)`, implicit rows, `place-items`, `1 / -1`), absolute boxes against their parent (`absolute.rs`),
and `overflow` (the flex automatic minimum of 0 and `content_size`; the scroll offset is `nui-ui`'s). `DrawList`
has `push_clip`/`pop_clip`, drawn by Direct2D as aliased clips. `demos/shell-boxes` now builds `#page` as a real
grid (tests/shell.rs unchanged) and has the gallery (`target/tmp/gallery.png`, 718 x 700). 28 flex, 14 grid and
11 position tests; the numbers are derived by hand from the page's CSS. Plex Mono's baseline at 1.125 em is
still UNVERIFIED, and the gallery's figures, durations, stage height, cut line top and biome names are
PLACEHOLDER. Not built: grid rows, areas, spans other than 1 and `1 / -1`, bare `1fr`, `position: fixed`.

Two observations read from the page's CSS cascade, not checked in a browser:

- `#details` also carries the class `choice`, so it inherits `gap: 4px` and `align-items: baseline` (line 147):
  its buttons are 3 px apart despite `margin-left: -1px`, they do not share hairlines.
- `.ore-row.head` is 11 px, so its `6ch` / `5ch` columns are narrower than the rows' (39.6 / 33 against 43.2 /
  36): the head words sit about 10 px right of the fields' left edges, and about 6 px right of their right edges.

Build plan step 9 is done: `crates/nui-ui` (ids by FNV-1a over the key path, left-button pointer input, the page's dark
and light themes, label, button in three looks, toggle; hit tests use the last frame's rects) and `demos/widgets`
(window, `--shot`, and the sheet of states in `docs/shots/widgets-dark.png` and `widgets-light.png`: the first
pictures in the page's own looks). `DrawList` text has a weight (Regular, SemiBold), so `#generate` is drawn in
Plex Mono 600. Measured: the Plex Mono ascent at 100 px puts the ink bottom of "H" at 113 against the expected
112.5, so ASCENT_EM = 1.025 stands (within the 2 px tolerance). Still UNVERIFIED: DESCENT_EM = 0.275, and the
underline's position (3 px under the baseline) and thickness (1 px) against a browser. The page has no :active
look, so a held widget looks hovered. Disabled widgets ignore the pointer, whereas the browser still matches
:hover on disabled `.choice` buttons: a known small difference. No transitions, cursor shapes or keyboard yet.

Build plan step 10 is done: `nui-ui` has `fold`/`fold_with`, `tab` (strip and choice looks), `detail_tab`, `strip_gap`, `number`
(no typing yet) and the styles of the strip, choice and details rows; text looks have a letter spacing, paint order follows
CSS (positioned boxes last) and `mix` is public. No slider was built: neither application has one. `demos/widgets
--controls` shows them live and `docs/shots/controls-dark.png` and `controls-light.png` are the sheet (12 new `nui-ui`
tests with the page's numbers, 2 picture tests). Two observations read from the page's CSS, not seen in a browser:

- Hovering the pressed strip tab: `.strip button:hover:not(:disabled)` (specificity 0,3,1) beats `[aria-pressed=true]` (0,2,1),
  so the word turns `--text` on a `--text` ground and vanishes. `nui-ui` keeps the pressed word on hover on purpose.
- The comments at preview.html line 657 and launcher line 571 still say "slider", and `--track` is used by no rule.

Build plan step 11 is done: `nui-base` has `Event::Key`, `Event::Char`, `PointerDown.clicks`, `Key`, `Mods` and the `Clipboard`
trait; `nui-win32` turns `WM_KEYDOWN`, `WM_CHAR` (with surrogate pairs) and click counting into them and has `SystemClipboard`
(CF_UNICODETEXT; the manual round-trip test `cargo test -p nui-win32 --test clipboard -- --ignored` passed); `nui-ui` has the
editing model (`edit.rs`), `text_field` (Text and Compact looks) and the editable `number` with a `NumberSpec`, one focused
field with Tab order = build order and `Ui::want_redraw`. `demos/widgets --fields` is the live mode and
`docs/shots/fields-dark.png` and `fields-light.png` are the sheet (24 `nui-ui` field tests, 10 edit-model unit tests, 2
picture tests). Unverified, read from the page or chosen by us, not seen in a browser:

- The selection colour (`#0078d7` on `#ffffff` text): Windows' defaults; the page has no `::selection` rule.
- The caret and selection height, 15.6 px, is ascent plus descent (1.025 + 0.275 em), not the font's measured line gap.
- The intrinsic width of a text field with no width, 20 chars (the browser's `size=20`).
- A pasted line break or tab is removed; Chrome may turn it into a space.
- `OpenClipboard(None)` is used when no window exists (tests); the docs warn that setting may then fail. The ignored test
  passed here, so it does work on this machine.
- The caption of script row 9 was cut to "type the quick brown…": the longer one ran under the field.
- The second number of the live mode is labelled with demo wording ("Elevation boost"), and the controls sheet's "Land
  against ocean" row uses `ocean_offset`'s spec (-1..-0.2, step 0.05), not `continents_scale`'s.

Build plan step 12 is done: `nui-ui` has `ellipsize` and `ellipsis`, a vertical scroll area (`scroll`, `scroll_style`:
thumb, 11 px gutter, bottom fade, wheel, thumb drag), hit tests cut to the visible part of a widget, `list_row`,
`list_head` and `trend_row`; `demos/widgets --lists` is the live mode and `docs/shots/lists-dark.png` and `lists-light.png`
are the sheet (11 `nui-ui` tests in `tests/list.rs`, 1 `Rect::intersect` test, 2 picture tests). Unverified, chosen by us
or read from the page, not seen in a browser:

- `GUTTER` 11 px (what the thin classic scrollbar reserves), `THUMB_MIN` 18 px and `WHEEL_STEP` 100 px per notch.
- Plex Mono's U+2026: zoomed 5x in the sheet it looks like three dots in one cell at the baseline, in the text's grey, so
  it seems to be the font's own glyph and not a fallback; nothing proves the font has it.
- The underline's thickness, and that a 1 px band per row quantises the page's mask gradient well enough.

Two observations read from the page's CSS, not checked in a browser:

- When the scrollbar shows, the rows' right edge moves 11 px left of the legend's: the legend is outside `.scroll`, and
  the 10 px padding only compensates the -10 margin.
- The legend's `ch` is 11 px, so its min widths are 52.8 and 39.6 against the rows' 57.6 and 43.2. The right edges
  still align because the cells are right-aligned.

Build plan step 13 is done: `demos/preview` is the world preview's shell (toolbar, three columns, the 43 real
settings in their five groups, static values), `preview [--light] [--size WxH] [--shot out.png]`, with
`docs/shots/preview-1400.png`, `preview-1200.png`, `preview-1000.png` and `preview-1400-light.png`. `nui-ui` gained
`wrap_lines`, `paragraph`, `fill`, `select`, `TextLook::small` and `label`. Tests: 7 in `nui-ui/tests/shell_bits.rs`,
4 data, 10 layout and 4 picture tests in `demos/preview`. The columns' rules and Export's right edge match a capture
of the page (`target/tmp/page_<width>.png`, the file opened without its server) to the pixel at the three widths.

Steps 8 to 13 ran as one workflow (22 agents, 2 h 07 min): for each step an Opus agent set the scope and checked
the pictures, a Sonnet agent wrote the code. The session then reran `cargo test --workspace` (green), checked the
31 commits (author Louis only, no trailer) and looked at `preview_1400`, `preview_1000`, `preview_1400_light` and the
page capture itself.

- PLACEHOLDER: the map slot (a --line square; step 14 paints the map there), the seven figures, and the 8 band widths and shares (step 18 computes them).
- UNVERIFIED: the select's value text place (7 px from the left, centred); the reference capture's select was empty, so only its height (23) and arrow were measured. The page capture has no data, so nothing inside the settings, the run block or the list was compared with a browser.
- Not built: changed marks (` *`, `pack X`, `N changed`, Save and naming), hints and tooltips, the select's menu, the ores group, `#note`, `#filter`, the tabs that switch lists (only their pressed state changes), the map, runs and progress, and export.
- Live: folds, tab presses, the detail choice, Pack, typing in the seed and the number fields, the toggles and the wheel.

Next: step 14 (the 2D map view).

Provisional choices, made by the session on 2026-11-14 and not settled: the maintainer had no view of their
own and asked to see the result first, so revisit both once the panels can be looked at.

- No height key over the 3D view: the height list now exists in the right column of demos/preview (swatch, band, width, share); putting it beside the 3D view is for the step that joins the two.
- The overlay's facts move into a side panel with a hairline frame, beside the view as on the page; until
  then the filled plate stays, because grey text without a ground is unreadable over bright snow.

## Goal of this run

Replace the world preview's three.js view (`..\world-preview\page\preview.html`, `refresh3d()` near
line 1842) with a native Rust program, through a small UI kit of our own. Done for this run means: the
`relief` demo draws the real terrain from `C:\Games\preview\out` in 3D, in a Win32 window and as a `--shot` PNG,
and the PNG has been looked at and compared with the web preview's look.

## Decisions already taken (do not reopen)

- Direct3D 11 + Win32 directly. No tao, no wry, no web page, no Slint, Electron or Tauri.
- Only `nui-win32` names Windows; the other crates are portable so a Linux backend can replace it later.
- No third-party crate except `windows = "=0.62.2"` (already in the cargo cache), and only in `nui-win32`.
  Open-source libraries (Taffy, microui, egui, Clay) are read for how they do things, never depended on.
- PNG, lossless, our own encoder (`nui-png`, done): hairlines and text must not blur, so no JPEG or WebP.
- Reversed-Z infinite perspective (clear depth 0, compare GREATER) instead of three.js's logarithmic depth.
- Consolas first; IBM Plex Mono (embedded woff2 in preview.html, SIL OFL) comes after step 6.
- Work happens on `main` in this young repository, which has no remote: nothing to push.
- Commits under the maintainer's name only: no `Co-Authored-By`, no "Generated with" line.

## Where it stands

- Done and committed (`692a87a`): `nui-base` (Color, Vertex, Material, the `Gfx` and `App` traits, Event,
  math with column-major `Mat4`, `perspective_reversed`, `look_at`, the `Orbit` camera, `DrawList`, the log
  hub with `observe!`) and `nui-png` (encoder with filter heuristic and fixed-Huffman deflate, plus a decoder
  of our own files for tests). 8 tests green.
- Drafted, never compiled, untracked: `crates/nui-win32/` with `Cargo.toml`, `src/lib.rs` (`shot()`) and
  `src/renderer.rs` (device with WARP fallback, swap chain or offscreen texture, D2D on the DXGI surface,
  DirectWrite text, shaders compiled at start with `D3DCompile`, STAGING readback). `src/window.rs` (the
  `run` function it exports) does not exist yet. Expect compile errors against windows-rs 0.62.2 signatures.
- The workspace `members` in the root `Cargo.toml` list only `nui-base` and `nui-png`; add `nui-win32` and
  `demos/relief` as each one comes to exist.

## Steps left

1. **Offscreen renderer** (`nui-win32`): compile `renderer.rs`; a test or tiny example clears to a colour,
   draws one flat triangle and a `Frame`/`Text` command, writes a PNG. Pass: the PNG shows them.
2. **Window** (`window.rs`): `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`, a Win32 class and window,
   events mapped to `nui_base::Event` (size, pointer, wheel, close, `WM_DPICHANGED`), `GetMessageW` when idle
   and `PeekMessageW` while `App::frame` asks for more, `Renderer::resize` on `WM_SIZE`.
3. **Relief demo** (`demos/relief`, binary `relief <data-dir> [--shot out.png --size WxH --scale S]`):
   - reads `preview.json` (seed, samples 256, step 16, originX/Z, seaLevel 63, minY, maxY), `heights.bin`
     (Int16 LE, samples²) and `biomes.bin` (Uint16 LE). Parse the JSON by hand: no serde.
   - paints the Relief texture like `paint()` near line 1379 of preview.html: `slope = (east - west) +
     (south - north)`, `shade = clamp(1 + slope * 0.012, 0.75, 1.25)`, colour from `heightColour(y, sea)`:
     stops `[[0,[110,40,42]],[40,[80,40,50]],[90,[40,38,48]],[150,[25,20,45]],[210,[0,0,70]],[260,[0,0,96]]]`
     in HSL (check the exact interpretation in preview.html, `hslToRgb` near line 1035), above the last stop
     `[245,245,245]`, underwater `hsl(212, 55, 46 - d*26)` with `d = min(1, (sea - y) / 60)`.
   - mesh: vertex `(col*step - side/2, height, row*step - side/2)`, uv `((col+0.5)/n, (row+0.5)/n)`, texture
     sampled nearest. Water quad at `sea - 0.5`, colour `#2f5c9e`, alpha 0.25. Ground `#000`.
   - camera: `Orbit` with eye `(0, 1.1w, 1w)` where w is the side, fov 50°, near 10; drag turns, wheel zooms,
     pitch kept under π/2 - 0.02 as OrbitControls does.
   - a small overlay in a hairline `Frame` (line `#2a2a2a`, text `#ececec`, muted `#8a8a8a`), its strings in
     one `text` catalog.
4. **Review**: take `--shot` pictures at 1280x800 scale 1 and 2560x1600 scale 2, open them with Read, compare
   with the web preview's 3D view (served by the launcher, or reason from `refresh3d()`), fix what differs.

Data for tests: `C:\Games\preview\out` (seed 94427382). Check it exists before relying on it.

## Workflow shape

The steps depend on each other, so the workflow is a chain, not a fan-out; keep it under 10 agents.

- Each step: one implementing agent (Sonnet) that writes the code, runs `cargo build` and `cargo test` in
  `rust-native-ui`, and commits once it compiles (small commits, `feat:`/`fix:` messages in
  English); then one checking agent (Sonnet) that rebuilds, runs the step's pass check, opens the PNG and
  answers pass/fail with reasons. On fail, the implementer gets the reasons back, at most two more rounds.
- The orchestrator (the session) reads each verdict, looks at the final PNGs itself, and reports in French.
- Every agent prompt carries the rules of `AGENTS.md` and the commit rule above.

