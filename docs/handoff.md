# Handoff: steps 0 to 6, to run as a workflow

Written 2026-11-14 at the end of the session that designed the kit. The next session starts here and launches
a Workflow (the maintainer asked for one in words: "le mieux, ce serait de lancer un workflow", so the opt-in
is explicit). Read `AGENTS.md` and `docs/design.md` first; this file says where the work stands and how to
finish steps 0 to 6.

## Status: steps 0 to 19 done (2026-11-15)

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

Build plan step 17 is done: data and runs (`docs/shots/run-progress.png`, `export-done.png`). `preview` reads
`--data` (default `C:\Games\preview\out`; without a `preview.json` there it starts with no world, as the page shows
`noPreview`; an explicit `--data` that fails exits 1) including `run.json` (`world::Made`: the seed and the 43 values the
map was made from, which fill the form: `PreviewState::for_world`, the page's `loadWorld`). Generate starts the
generator given by `--generator` (no default: without it the status says the run failed) as serve.ps1 does:
`powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& '<generator>' -Dir … -Out … -Seed '…'
-Samples N -Detail D -Tectonic @{ 'key' = v; … }"` (all 43 keys, switches as `$true`/`$false`, `-Pack` only with
`--pack`), after deleting `coarse.json`, `progress.json` and `partial.json` from the out folder, with stdout and stderr in
`run.log`. `--runs DIR` (default `target/tmp/runs`, refused with exit 2 unless inside this repository's `target/tmp`)
holds `out/`, `server/`, `run.log`, `pace.json` and `exports/`: nothing a run or an export writes goes anywhere else,
and the data of `--data` is never written. The app reads `<out>\progress.json` (`{pass, step, done, total}`) once a
second (the first time one second after the start), and `Follow` (the page's `follow()`) turns it into the done steps
with the second each ended at, the step under way with its percent, the hairline's share and the status line. The
run block has the page's looks: the generate button `Running { share }` (frame in `--text`, a `--line` fill from the
left, "Generating…"), the 34 x 34 stop square beside it (`Ui::stop_button`), the status ("20 s of about 50 s.", the
first-run "measuring" line, "Preview ready in …", "Stopped. The map is the one from before.", the failure and the
refused seed in the warning tone, "The map shows these settings."), the detail tabs' small wait line from the learned
pace (`run::Pace`, the page's `timing` kept in `pace.json`, learned from `timing.json`'s `workMillis`), `#steps` under
the map (the hairline and the line of ✓ steps and the step under way), the busy veil over the frame (the ground at 0.7
over the map, the 3D view or the cut, not over the note and the lines under), and the queued re-run (a setting moved
during a run: the button offers a new run, which stops this one and starts by itself). A finished run (the generator
ended with a status of 0 and `preview.json` in out) is read again as the world, the form takes its settings, the pace
learns. Stop is `taskkill /PID … /T /F` through `nui_base::Launcher` (new, no OS code; `nui_win32::SystemLauncher`
implements it with `std::process` and `CREATE_NO_WINDOW`, no new `windows` feature): it takes the generator's whole
tree, the game server included; the app does the same when it is dropped (serve.ps1's `finally`). Export writes
`biomes.png`, `flat.png` (not for a coarse map), `height.png`, `temperature.png` (each the whole map, whatever is
chosen: `paint()` encoded as is) and `section.png` (the last uploaded sheet, alpha kept) into
`exports/<seed>_<UTC stamp>/`, then shows "Exported to <folder>" with "Open the folder" (Explorer through the launcher)
and "Close" under the map.

Tests: `tools/stub_run.ps1` stands for `preview.ps1` (same parameters, same deleted files, `progress.json` written
atomically, `timing.json`, `run.json`, the same throw when killed; modes `finish`, `hold` and `fail` by
`$env:NUI_STUB`; it refuses any Out or Dir outside `target/tmp` before it creates anything and downloads nothing).
`tests/run.rs` R1 to R7 drive the whole application against it offscreen (R1: the 40 % picture
`target/tmp/run_40.png` and the stop that ends the stub's grandchild process; R2: the finished run becomes the map and
the pace learns 1.5 s of work; R3 failed, R4 refused, R5 queued, R6 dropped while running, R7 the guard), `tests/export.rs`
E1 to E4 (E1: the four PNGs equal `paint()` byte for byte and 200 stage pixels of `target/tmp/export_done.png` each equal
a texel of `biomes.png` around their place; E2 a chosen biome is not exported; E3 `section.png` is the sheet; E4 the
notice's two links, through a recording launcher), `nui-win32/tests/process.rs` X1 (stop takes the grandchild),
`nui-ui/tests/run_bits.rs` N1 to N3, 10 unit tests in `run.rs` and `text.rs` (the command line, seeds, the pace, the
follow, durations, folder names, `same_as`, the progress file) and `tests/data.rs` d5 (`run.json` of the real folder). The
workspace has 305 passing tests, none skipped. The step-13 test P5 (generate disabled) and the section CLI shot test
followed the page: with no map Generate is enabled and filled; the CLI picture is built from `PreviewState::for_world`.

- PLACEHOLDER: the figures and the band widths and shares of the right column (step 18), unchanged; nothing new.
- UNVERIFIED: **the real generator was never run.** `preview.ps1` was read, not executed: it downloads the Fabric server
  jar and the pack tools, may build the mod with gradle inside `world-preview\mod` (writes `mod\build`), needs a pack
  (`WORLD_PREVIEW_PACK` is not set on this machine), the Minecraft EULA and Java 25. Only its protocol, read from
  serve.ps1, preview.ps1 and Progress.java, is followed, and tested with the stub. That the command line works in the real
  script (quoting of a path with spaces, the hashtable), that `-Samples` means what the page means by `samples` for it,
  and the 0.5 s atomic rename on Windows are therefore unproven. Also: the ✓ glyph (it drew as a check-like sign in the
  picture, DirectWrite may be falling back from Plex Mono), the stop icon's half-pixel place (13.5 px), the export
  folder's UTC stamp (the page uses local time), the seed check being ASCII only (serve.ps1's `\w` takes every script's
  letters), the links' 1 px of vertical padding (the page's `.saved button` has none), and the run's last step after a
  finish: the look may or may not have caught the extras pass, so the last done step is the map or "Cuts, ores and
  structures".
- Not built: sketches and drafts during a run (`coarse-*.bin`, `partial-*.bin`: the map stays the old one, veiled, for
  the whole run), `#numbers.stale`, joining a run found under way on opening (`/api/preview/run`), the close
  confirmation (`beforeunload`), the tray icon, `tell()` to a host page, the area select's menu (samples stay the loaded
  world's, 256 by default), the tooltips and titles, the history and `remember()` (step 18), `export.json` (it needs the
  figures, step 18), `structures.png` (it needs the 2048 px marks rasteriser), and the `*-3d.png` pictures (they need a
  second offscreen 3D render at 1600). The "Pictures: view" steps of an export are not shown one by one: it is
  synchronous. The window redraws without rest while a run is under way (a wake timer is step 19's).
- Choices where the brief was silent: the stub's `finish` copies `heights.bin`, `biomes.bin`, `section*` and the JSON
  files named in the brief and nothing else; `Want` is recorded by the shell and acted on inside the app's build loop (up
  to four builds a frame, so the frame shows the result of the press); the generate button is disabled while the stop is
  being taken whichever look it has, as the page disables both; the `reload` field of the brief was not needed (a finished
  run uploads the meshes inside `poll`, before the frame begins) and is not there; `Link` buttons are 12 px already, so no
  smaller look was added; the veil is painted before the note so the note stays undimmed; the unit tests of `run.rs` write
  in `target/tmp/run_unit*`; R1 reads the button's fill 4 px under its top edge (mid-height would meet the word's
  glyphs); `--runs` and the generator path are made plain (without `\\?\`) because the stub's guard compares plain paths.

Build plan step 18 is done: the right column follows the view with the page's real numbers, the history is kept, and the
theme follows the system. Built:

- `nui-base`: `Cmd::Polyline` (an open stroke, round joins, flat ends) and `Event::Scheme { dark }`. `nui-win32`: the
  polyline as a Direct2D path geometry with one stroke style (`D2D1_LINE_JOIN_ROUND`, flat caps), `system_dark()` (the
  registry's `AppsUseLightTheme`; missing or an error is light, as Chrome reads it; the only new `windows` feature is
  `Win32_System_Registry`) and `WM_SETTINGCHANGE` ("ImmersiveColorSet") turned into `Event::Scheme` when the value differs
  from the last one sent. `nui-ui`: `hover_area`, `list_part` (the `.part` heads), round mark swatches (`RowOpts.mark`),
  `ButtonLook::Pick` and `PickNone`, a strike-through decoration, and `trend_row` with a plot drawn in its slot.
- `demos/preview`: `figures.rs` is the page's `measure()` (zones, the 10 figures, the five lists, `Math.round`,
  `toFixed(1)`, `percent`, `blocks`) and the cells of `showFigures`, with the change column against the run before.
  `history.rs` is `remember`, `runsHere`, `changes` and the trend lines of `showHistory` as numbers and path text, kept
  in `<runs>/history.json` (the page's localStorage `preview.history`, the page's JSON shape) and `<runs>/before.json`
  (`preview.before`). The shell's right column: the figures with their change, the history fold (closed at first; the
  pointer over it picks the nearest run), and the list that follows the view (biomes by share then "Not in this area",
  the height, ground and temperature bands in order, structures in two parts with round marks and their part heads),
  rows that mirror the choice, a filter, and `#pick`. The window follows the system's scheme live (`--theme
  dark|light|system`; the default is the system for a window and dark for `--shot`).
- Evidence: `tools/page_figures.mjs` runs the page's own `measure`, `listOf`, `showFigures`, `remember` and
  `showHistory` in Node on `C:\Games\preview\out`, and `tests/page_figures.rs` finds, bit for bit: the 10 figure fields (flat
  25.929, expanse 670.33, 25 biomes of 136 ...), the 5 lists (25, 3, 8, 5 and 75 rows: ids, names, colours, cells and
  widths), the 111 absent ids, the 232 width and share strings, the 3 `showFigures` cases (22, 25 and 22 cells; the
  second holds "=", "+1.0 pt", "−0.3 pt", "−200", "≈" and "+3"), the 5 stored history entries (43 settings each, written
  back and read again equal) and 3 trend cases (15 path strings equal, with a gap in a line and the probe words).
  `tests/figures.rs` (G1 to G7, H1 to H4, T1 and the command line) checks the column through the application and the
  offscreen renderer: a theme switched live equals a fresh light picture byte for byte. 341 tests in the workspace, none
  skipped. The shots are `docs/shots/figures-1400.png`, `figures-light.png`, `figures-change.png`, `figures-1000.png`,
  `history.png` and `structures-list.png`; the step 13 to 17 shots stay as their record.

- PLACEHOLDER: none left in the right column.
- UNVERIFIED: the strike-through's place (a 1 px fill 0.3 em above the baseline) and the pick links' hover; that `#pick`
  wraps only between its pieces (the page's inline text may break inside a name); the polyline's joins and the history
  dot's ring against Chrome's SVG; the `WM_SETTINGCHANGE` path was not exercised live (the registry read is tested, the
  window message is not); `num_digits` on exact binary halves (Intl may round on the shortest decimal); a figure that is
  NaN (an area with no land) is shown as a dash, where the page would write "NaN".
- The history pictures use fixture runs written by the tests (three entries of the real seed with one setting moved
  each); no real second run exists on this machine.
- Not built: `export.json`; `#numbers.stale` (no sketches exist natively, so it never applies); the tooltips and titles
  (definitions, "needs the full detail", "before: ...", the part hint, the rows' ids); cursor shapes (the history's
  crosshair); the fold's open state kept across launches; the window title bar's dark mode (DWM); history eviction (there
  is no quota natively: the file grows by about 1 KB a run); line wrapping inside a `#pick` piece.
- Choices where the brief was silent: the first two commits of the brief became one (nui-base alone would not compile
  with the renderer's exhaustive match), so the renderer's polyline is in the first and the registry in the second;
  `BAND_FIGURES` went with the shell in commit 7, since the shell used it until then; `ids.rows` was already the
  settings' rows, so the list's are `list_rows`, `list_keys`, `list_parts`, `list_part_words`, `absent_rows`, and
  `band_scroll` became `list_scroll`; with no map the list's title is empty (the page's markup before any list), and the
  first part head is "first" by its place in the page's list, hidden or not; a part head that the filter empties is
  not built; the history is cloned with the state at every build (about 43 numbers a run; fine for hundreds of runs);
  `PreviewApp` remembers a map in `new()` (in memory) and again in `with_runs()` after reading the file, so a window
  with runs does not duplicate the last entry; the `before` figures are written before the new map is read, only when
  the run finished; `Figures::get` treats NaN as missing; the history entry takes its settings in `SETTINGS` order, a
  page's file keeps its own order.

Steps 14 to 18 ran as one workflow (17 agents, 2 h 40 min; step 17 needed a second round, the others passed in their first): for each step an
Opus agent set the scope and checked the pictures, a Sonnet agent wrote the code. The session then reran
`cargo test --offline --workspace` (green), checked the 39 commits (author Louis only, no trailer, clean tree),
that `C:\Games\preview\out` and `world-preview` were untouched, the one new `windows` feature (`Win32_System_Registry`,
inside `nui-win32`), and looked at `figures-1400`, `history`, `stage-3d-relief`, `section-1400` and `run-progress`.
The step 15 and 17 shots still show step 13's placeholder figures in the right column: they are those steps' record.

Step 19 (2026-11-15) answered the maintainer's remark that the page had too many breakpoints: the world preview's
panels never move (left 320, right 330, gap 28, padding 18/28/22/28), the centre is one column that scales by one
rule (the map's square side is `min(W - 762, H - 144)`; the brief's `W - 752` left ten pixels out: 320 + 330 + 2 x 28 +
2 x 28 = 762), and the top bar has one form from 1000 px up. The window has a minimum of 1000x600, so no layout
below it exists. The bar is the launcher's, on a frame of our own (`WM_NCCALCSIZE` returns 0, `WM_NCHITTEST` asks
`App::hit`, the buttons are drawn by `nui-ui`'s `chrome.rs`), with the tool switch Home / preview. The same step
added cursor shapes, tooltips (the page's titles, 0.5 s delay, painted last), a wake timer with no busy loop, the
occlusion callback (a covered window is not drawn but still follows its run), device loss through
`DXGI_STATUS_OCCLUDED` and a rebuild, a DPI change through `WM_DPICHANGED`, `export.json`, and errors in the page's
wording: a preview folder that is missing shows `noPreview`, one that cannot be read says so in the status line and
the window stays open (the CLI no longer exits 1 for it).

- Evidence: `cargo test --offline --workspace` gives 380 passed, 0 failed, 1 ignored (the old clipboard test, which
  touches the real clipboard). Pictures in `target/tmp`, kept in `docs/shots`: `bar-1400.png`, `bar-1400-light.png`,
  `bar-x2.png` (no x1.5 bar picture is kept; `bar_1400_x15.png` stays in `target/tmp`), `bar-home.png`,
  `bar-maximized.png`, `bar-hover-close.png`, `error-missing.png`, `error-unreadable.png`, `responsive-1920.png`,
  `responsive-1400x600.png`, `responsive-section-1000.png`, `icons-x1.png`, `icons-x1.5.png`, `icons-x2.png` (the 16 icons at 16 DIP, written by
  `tests/polish.rs`), `tooltip.png` (the Relief tab's title under the pointer). The polish tests are in
  `demos/preview/tests/polish.rs`: P1 (a lost device, then a new renderer: the same bytes as a fresh app), P2 (a move to
  scale 1.5: the same bytes as a fresh 1.5 renderer), P3 (a run under way: `frame` asks for no other frame, `wake_in`
  within 1 s, `tick` reads `progress.json` with no renderer).
- PLACEHOLDER: `HOME_LATER` ("The launcher's home comes with step 21.") and the Home pane it fills, which is empty
  otherwise; Home's own content and its one switch (two parts side by side or one above the other) belong to step 21.
- New `windows` features, all inside `nui-win32`: `Win32_Graphics_Dwm` (`DwmExtendFrameIntoClientArea`, to keep the
  window's shadow on the own frame) and `Win32_UI_Controls` (`TrackMouseEvent`, so that the pointer leaving the window
  is known). `RegisterOcclusionStatusWindow` needed nothing new (`Win32_Graphics_Dxgi` was already on).
- Report for the maintainer: in the pack's repository, `git status` shows ` M pack/pack.toml` with an empty content diff;
  only its modification time (2026-11-15 00:11) differs. Nothing in this work touched it (that repository is read
  only here); the likely cause is a launcher (webview) run in that checkout, since ignored files under `run/app/webview/EBWebView`
  were also written between 00:11 and 00:18. The maintainer should confirm; `git update-index --refresh` there would
  clear it, and was not run.
- Choices where the brief was silent: `Cmd::Shape` (a path with the winding rule) was added to `nui-base` for the
  icons, drawn crisp with its vertices on device pixels; the icon grammar is the reduced SVG one (`M m L l H h V v Z z`);
  a hidden window's `tick()` looks at the run and leaves the end for the next frame, which holds the device;
  `export.json` writes `ores.added` as null (no editor for the added rows natively) and `ores` as null outside the
  section view; the Home pane is a placeholder text; `UNREADABLE` is our own sentence in the page's style.
- UNVERIFIED: the bar's hairline at scale 1.5 spills one device row below the bar and was not compared with Chrome;
  no live check of the window (the cursor, the tooltip delay, the minimum size, occlusion, a DPI change between monitors,
  Aero snap on the own frame) was made, only the pure functions behind them and the offscreen pictures.
- Also UNVERIFIED: the DWM margins of 1 px on a live window (a system-coloured line at the top would mean using 0),
  the maximise inset, `WM_NCMOUSELEAVE`, OrbitControls' cursor, the tooltip's look (ours, in the page's tokens), and the
  `ores.added` default.
- Not built: glyph tabs and `condense`, the options transition, the marker blink, snap layouts on Maximise, the close
  confirmation, `#account`, caret blink, focus rings, changed marks and their hints; the 3D and structures pictures in
  the export; the window's dark title bar (the own frame has none); IME.

Next: step 20 (IME in the text field, UNVERIFIED need) and step 21 (the launcher's own migration, in the pack's repository).

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

