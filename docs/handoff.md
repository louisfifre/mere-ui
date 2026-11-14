# Handoff: steps 0 to 6, to run as a workflow

Written 2026-11-14 at the end of the session that designed the kit. The next session starts here and launches
a Workflow (the maintainer asked for one in words: "le mieux, ce serait de lancer un workflow", so the opt-in
is explicit). Read `AGENTS.md` and `docs/design.md` first; this file says where the work stands and how to
finish steps 0 to 6.

## Status: steps 0 to 16 done (2026-11-14)

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
- Not built: changed marks (` *`, `pack X`, `N changed`, Save and naming), hints and tooltips, the select's menu, the ores group, `#filter`, the tabs that switch lists (only their pressed state changes), runs and progress, and export. (`#note` and the map came with step 14.)
- Live: folds, tab presses, the detail choice, Pack, typing in the seed and the number fields, the toggles and the wheel.

Build plan step 14 is done: `preview --data C:\Games\preview\out` draws the 2D map in the stage, in a window and as `--shot`
(`--view`, `--biome`, `--structure`, `--look` are the page's `?view=`, `&biome=`, `&structure=`; `docs/shots/map-biomes.png`,
`map-flat.png`, `map-relief.png`, `map-temperature.png`, `map-structures.png`, `map-biomes-light.png`,
`map-relief-zoom8.png`). Built: `nui-base` has `Cmd::Image`, `Disc` and `Ring`; `nui-win32` draws them in Direct2D
(a premultiplied BGRA bitmap beside each D3D texture; nearest-neighbour); `nui-ui` has `canvas`, `Resp::pressed`,
`pointer` and `take_wheel`; `demos/preview` has `world.rs` (the out folder with flat land, structures and the cut),
`colour.rs` (the page's colours and a regex-free matcher), `paint.rs` (`paint()` for the five views, the choosing dimming,
the marks), `map.rs` (look, wheel, carry, grid, cross, ticks, points under the pointer), `app.rs` (`PreviewApp`),
`json.rs` (relief's copy; step 15 leaves one) and `tools/page_paint.mjs`. `paint()` is byte-identical to the page's own
code run by Node on `C:\Games\preview\out`, for 12 cases (five views x dark and light, a chosen-biome and a chosen-height
case); the biome and structure colours, the structure order and the flat count are equal too. Through the window path
the rest look shows each point as its texel, and at 8x zoom every block interior equals its texel exactly with 929
sharp boundaries (so Direct2D's nearest-neighbour is exact at a fractional scale). Tests added: 1 in `nui-base`, 4 canvas tests in `nui-ui`, 1 in `nui-win32`, in `demos/preview` 7 unit tests
(world, look), 6 colour tests, 1 page comparison, 4 map and 5 interaction tests.

- PLACEHOLDER: the 3D and section slots (a `--line` fill; they draw in steps 15 and 16), the figures and band widths of the right column, and the list there, which still shows the height bands whatever the view.
- UNVERIFIED, read from the page's CSS and not seen in a browser: which pixel a grid line lands on when its place is fractional, the cross's halo (two `drop-shadow(0 0 1px #000)` approximated by a 50 % black border of one pixel), the tick's row rounding, Chrome's pixelated sampling at a non-integer scale, the marks (one stroke per ring instead of one path per kind, so overlapping rings darken twice; the land under them is drawn nearest where the page smooths its 2048 px canvas), and the centring of a wrapped `#note` (its block is centred, its lines are left-aligned).
- Not built: the `#nav`, `#navFrame` and `#navNote` of the cut (step 16); the list switching with the view and its rows mirroring the choice, the `#pick` line, the tooltips (the `needsFull` title) and cursor shapes; `openGroupsFor`; sketches and drafts (`coarse-*` and `partial-*` files), the busy opacity, export and the 3D view. A click on the map chooses (the dimming), but nothing beside the map shows the choice yet.
- Choices where the brief was silent: a biome without a `temperature` reads 0; a point whose biome index is out of range is an error; `Look::wheel` treats a fraction of a notch as one step; the drag is read until the press ends, so the last movement before the release counts; `PreviewApp` also has `ui_mut` (for the clipboard); `ShellIds.probe_text` holds the probe's whole text because the line is cut with an ellipsis at 1400 px; commit 4 carries `colour.rs` because the world needs it, and commit 5 holds the paint, the map and the stage together because they depend on each other.

Build plan step 15 is done: the 3D view is in the stage (`docs/shots/stage-3d-relief.png`, `stage-3d-zoom.png`,
`stage-3d-relief-light.png`, `stage-3d-biomes.png`). `demos/preview/src/scene.rs` is the page's `refresh3d` as a
shared module (land and water meshes, `ground_at`, the orbit set-up with `?camera=`, `settle`, `upload`,
`water_alpha`, `draw`); `demos/relief` is now a thin client of it and of preview's `World`, and its `json.rs`,
`world.rs`, `mesh.rs` and `paint.rs` are gone (one copy of `land()` and `ground_at()` left). In the stage, a canvas
`relief` of side x side takes the left-drag (a full turn per side, as OrbitControls does with `clientHeight`) and the
wheel; the app draws the two meshes with the D3D viewport set to that canvas's rect, before the interface list, and
the toolbar's tabs repaint the draped picture (the camera is kept across views and 2D/3D). In Section the 2D/3D
tabs and the gap before them are not built. `preview --camera cx,cy,cz` is the page's `?camera=`. `Orbit` is `Clone`,
`Debug` and `PartialEq` because it lives in `PreviewState`.

- Evidence: before `paint.rs` was deleted, `relief`'s `paint()` against `preview::paint::paint(Height, dark, none)`
  on `C:\Games\preview\out` differed in 0 of 262,144 bytes; `relief C:\Games\preview\out --shot ... --size 1280x800` is
  byte-identical to `docs/shots/relief-1280.png` (0 differing pixels, same file hash). The CLI shot
  `preview --data C:\Games\preview\out --view height,relief3d --shot` is byte-identical to the test's `relief3d_1400.png`
  (same file hash), and `--camera 0.3,0.25,0.6` differs from it.
- Tests: `tests/relief3d.rs` (4) through `PreviewApp` and the offscreen renderer: S1 the square is (376, 56, 638,
  638), every pixel outside it equals the interface list drawn alone, 43.8 % of the square is not ground, Relief and
  3D are pressed, 2D is not, the square's corner is `--ground`, the probe has no "Ticks"; S2 after zooming in all four
  edge lines 1 px inside the square are 100 % terrain and nothing outside differs; S3 the light ground `#fafaf8`;
  S4 Biomes repaints (43.8 % of the square changes), 2D brings the map back, 3D brings back the identical orbit,
  Section leaves 6 tabs and no relief. `tests/interact.rs` O1 to O4 (yaw `2 pi x 100 / 638` within 0.01 and back, a
  press outside does not turn, one notch is 0.95 of the distance and the list's wheel does not zoom, a drag that
  leaves the square keeps turning). Also 5 unit tests in `scene.rs`. The workspace has 244 passing tests, none skipped.
- The D3D viewport alone clips the scene at the square's edges (S2 saw no leak), so no scissor rect was added to
  `nui-win32`. S2 asks for 30 notches, which the zoom range stops at its near end (`min_distance` = 40 steps).

- PLACEHOLDER: the section's slot (a `--line` fill; step 16), the figures and band widths of the right column, and the list there, which still shows the height bands whatever the view.
- UNVERIFIED: the glide's speed against OrbitControls' damping (`RATE` is ours), the 0.95 per wheel notch against three.js r160, nearest against linear sampling for the structures drape (the page draws its marks into a 2048 px canvas with linear filtering), and the live window (not looked at: only the offscreen pictures).
- Not built: structure marks in 3D (the Structures drape is `paint(Structures)`, the land under the marks, nearest); right-drag pan, middle-drag dolly, keys and touch of OrbitControls; cursor shapes, the busy opacity and the export of the 3D; the `no3d` note; resetting the camera when a new area of another width loads (step 17 brings runs); the list following the view.
- Choices where the brief was silent: `scene::settle` is relief's frame code unchanged; the drag scale is the canvas's `rect.h` (equal to the side); the 3D water alpha follows the view the drape was painted for; `--camera` is ignored without `--data`; relief keeps `scene::water_alpha(Height)` since it only shows the relief tab.

Build plan step 16 is done: the Section tab shows the cross-section in the stage (`docs/shots/section-1400.png`,
`section-1400-light.png`, `section-zoom.png`, `section-iron.png`, `section-1000.png`; `preview --data C:\Games\preview\out
--view section`, with `--cut N`, `--ore kind,..` and `--cut-look k,x,y`). `demos/preview/src/section.rs` is the page's
section model and ore play (rules, `ground_of`, `ore_play` with the page's `drawOf` stream, figures and the drawn
layer), `cut.rs` is the cut's look, its sheet, the strip and the frame on it, the probe; `text.rs` has the section's
wording and `num_digits`; `world.rs` holds `section: Option<Section>` (all four cuts read at load, 6 MB) and the 2D
ticks follow the cut being looked at (`tick_at(world, look, at)`). In the shell the frame is the column's width by
its height less `--under` (638 x 798 at 1400 x 940): the strip (56 px, the whole cut at its own range of heights, the
frame of the part looked at), the note, and the cut at true scale in one uploaded image of one texel a block, nearest;
the wheel zooms about the pointer (1.25 / 0.8 a notch, between half of the fitting scale and 12), a drag carries the
cut, a press on the strip brings the window there or carries it, the probe names the block under the pointer, "Another
cut" walks the four cuts and wraps. The right column shows ORES: the figures block is not built, the list has no rule
and no top padding, one row per ore with its low and high ground figures, and a click on a row picks the ore (the
cut then draws it alone, the other rows fade).

- Evidence: `tools/page_section.mjs` cuts the section block, the look, the drawing and the strip out of preview.html and
  runs them in Node on `C:\Games\preview\out`; `tests/page_section.rs` (5 tests) finds equal: the 25 rules (id, index, kind,
  playable, anchored), the 5 rows of mods and 0 unplayed rules; the ground of the 4 cuts (surface, floor, high, biomes,
  top, bottom, peak, crop, rows); the per-chunk figures of cuts 0 and 2 exactly, as f64 (iron cut 0: low 97.92265193370167,
  high 144.76666666666665), the plays' names and `counted` 512; the whole drawn layer of cuts 0 and 2 byte for byte (27,934
  and 32,501 ore cells); the strip's 35,728 pixels, its range (317, -30) and its note number; and three pictures of the cut
  (A: cut 0 fresh at 638 x 700, B: cut 2 at k 0.6 with iron chosen, so the ores are two blocks wide, C: cut 1 at k 4) with the
  settled look, the `drawImage` arguments, the sheet's texels and the frame's `left`, `width`, `top`, `bottom` all equal
  (1e-9 for the numbers). V8's `Math.sin` and `Math.cos` (fdlibm) and the MSVC runtime's gave the same veins on this
  data, so the 1-ulp fallback of the brief was not used. The play of one cut takes 0.17 s in a debug build, so no
  `opt-level` was set.
- Tests: `tests/section.rs` (12) through `PreviewApp` and the offscreen renderer: C1 the layout at 1400 x 940 (frame
  (376, 56, 638, 798), strip (376, 56, 638, 56), cut view at y 154, 638 x 700 and equal to the arithmetic box, under at
  864, six tabs, no figures, ORES with 8 rows, the probe's rest text); C2 300 cut pixels and 525 strip pixels each a texel
  of the picture under them or a neighbour, the sky is `--ground`, the frame's edges are `--text`, the hairlines are
  `--line`; C3 the wheel is `cut::wheel`, 30 notches out stop at the least scale; C4 a drag; C5 a press on the strip puts
  the frame's middle under the pointer, a drag of the frame moves it by the travel; C6 "Another cut" (cut 2 of 4 at z -992
  with a fresh look, four clicks wrap, the 2D tick of cut 3 at z -192 is drawn); C7 the iron row (no other ore colour in
  the sheet or the pixels, the other swatches fade); C8 the probe on rock, sky, ore, outside and on the strip; C9 a world
  without a section; C10 light (`#fafaf8`, `#171717`) and 1000 px (the note wraps, the cut view is 18 px lower); C11 at
  1400 x 600 the least scale is under 0.75 and the sheet is made with wide ores (`PreviewApp::cut_reach`). The CLI shot
  `preview --data C:\Games\preview\out --view section --shot` is byte-identical to the test's (98,081 bytes). Also 7 unit
  tests in `section.rs`, 9 in `cut.rs`, 2 in `text.rs`. The workspace has 279 passing tests, none skipped.

- PLACEHOLDER: the figures (the seven of the map) and the band widths and shares of the other views' right column, unchanged.
- UNVERIFIED, read from the page's CSS and not seen in a browser: Chrome's smoothing of the two squeezes the page's
  canvases undergo (the strip's 56 rows shown in 55, the cut's `H` rows in `H - 1`; here they are nearest, one image
  scaled by Direct2D), the strip frame's fractional edges (a `Frame` of one pixel at a fractional rect; the test only
  asks for at least one `--text` pixel at each edge), the probe during a drag (the page freezes it, here it follows the
  pointer), `num_digits` on exact binary halves (Intl may round on the shortest decimal), that the page's
  `getBoundingClientRect` of the cut view equals the arithmetic box in a browser, and that V8's sin and cos agree with the
  MSVC runtime's on other data than this one (they do on this one).
- Not built: the ores settings group on the left (editing the rows, Add, Without, Reset, `localStorage` `oreRows`,
  `veinOf` and the `everywhere` rows), the cut kept across runs (`keptCut`, `resumed`: step 17), the export of
  `section.png` (step 17), the tooltips (`oreFrom`, `oreColumnsHint`, `nextCutHint`, the `needsFull` title), cursor
  shapes, the busy opacity, the `#filter`, exact pixels at a device pixel ratio other than 1 (the pictures are made at
  DIP size), and `openGroupsFor`.
- Choices where the brief was silent: `cut::nav_frame` returns a `NavFrame` holding the page's four style values in f64
  (`rect()` gives the box drawn, with the 5 px minimum) so the Node comparison can be exact; `settled`, `wheel`,
  `dragged` and `sight` take the box as two numbers; `nav_go` has no `k` (it reads the look's) and `probe` takes the world
  (its section) and the `Shown`; `cut::nav_range` and `text::times` are the two small helpers the shell needs before the
  strip is drawn; the Node run has a third case C (cut 1, k 4, y -500) because A and B both sit at the frame's full height;
  `Section::load` reads the four cuts eagerly; a uniform with a bound missing is not a number (the page would go on to NaN);
  `PreviewApp::new` takes the named cut as the last one shown, so `--cut-look` is not reset by the first sync; the app
  reads the cut view's box from `ShellIds::cut_box` (the shell's arithmetic, asserted equal to the laid-out rect by C1 and
  C10) so the shell and the upload cannot disagree; the legend of the ore list shows only once a cut is shown, and the
  probe is empty until then; "Another cut" is built in the section view only; `--cut` is kept within the cuts and `--ore`
  names are not checked; a test also writes `target/tmp/section_none.png` (no section).

Next: step 17 (data and runs).
Provisional choices, made by the session on 2026-11-14 and not settled: the maintainer had no view of their
own and asked to see the result first, so revisit both once the panels can be looked at.

- No height key over the 3D view: applied in step 15. Nothing is drawn over the 3D in the preview (no key, no plate, no facts); the height list stays the right column's list, which does not follow the view yet.
- The overlay's facts belong to the side panel, which is the right column: applied in step 15 for the preview. The relief demo, a standalone window with no side panel, keeps its filled plate, because grey text without a ground is unreadable over bright snow.

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

