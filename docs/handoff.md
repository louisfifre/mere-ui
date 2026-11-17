# Handoff: steps 0 to 6, to run as a workflow

Written 2026-11-14 at the end of the session that designed the kit. The next session starts here and launches
a Workflow (the maintainer asked for one in words: "le mieux, ce serait de lancer un workflow", so the opt-in
is explicit). Read `AGENTS.md` and `docs/design.md` first; this file says where the work stands and how to
finish steps 0 to 6.

## Status: steps 0 to 21 done (2026-11-15)

The build run finished. `relief <data-dir>` draws `C:\Games\preview\out` in 3D in a window and as `--shot`; the
shots in `docs/shots` were compared with a capture of the page's 3D view (`world-preview/scripts/capture.ps1`,
`/?view=height,relief3d`) and match in framing, colours, water and edges. A review pass then added 4x MSAA
(the page asks WebGL for antialias), hairlines of one logical pixel, the page's biome total and wording,
OrbitControls speeds, a camera kept above the ground within a zoom range, and robustness fixes in the window
(no `&mut` aliasing across re-entrant calls, a cleanup guard, safe capture, device-loss rebuild).

Not exercised at runtime: device loss, capture lost mid-drag, a `WM_DPICHANGED` between monitors, and a
look at the live window's pixels (only its start, answer and clean exit were checked).

Later the same day: text is in IBM Plex Mono (`fonts/`, loaded by `mere-win32/src/fonts.rs` through
`IDWriteFactory5::UnpackFontFile`; the Pi cut's arrows and signs are reached by DirectWrite's coverage
matching inside the one family, seen in `tests/fonts.rs`'s picture, not a documented guarantee; the 600 face
is loaded but unused). `fonts/OFL.txt` was written from memory, as no copy was on the machine: compare it
with IBM's official LICENSE.txt before anything is published. Build plan step 7 is done: `mere-layout` (flex,
28 rect tests, dump) and `demos/shell-boxes`, whose pictures at 1400, 1200 and 1000 px match the page's
column widths; heights and the strip width the page does not fix are marked PLACEHOLDER there.

Build plan step 8 is done: `mere-layout` has a reduced grid (`grid.rs`: columns of a length, `auto` or
`minmax(0, Nfr)`, implicit rows, `place-items`, `1 / -1`), absolute boxes against their parent (`absolute.rs`),
and `overflow` (the flex automatic minimum of 0 and `content_size`; the scroll offset is `mere-ui`'s). `DrawList`
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

Build plan step 9 is done: `crates/mere-ui` (ids by FNV-1a over the key path, left-button pointer input, the page's dark
and light themes, label, button in three looks, toggle; hit tests use the last frame's rects) and `demos/widgets`
(window, `--shot`, and the sheet of states in `docs/shots/widgets-dark.png` and `widgets-light.png`: the first
pictures in the page's own looks). `DrawList` text has a weight (Regular, SemiBold), so `#generate` is drawn in
Plex Mono 600. Measured: the Plex Mono ascent at 100 px puts the ink bottom of "H" at 113 against the expected
112.5, so ASCENT_EM = 1.025 stands (within the 2 px tolerance). Still UNVERIFIED: DESCENT_EM = 0.275, and the
underline's position (3 px under the baseline) and thickness (1 px) against a browser. The page has no :active
look, so a held widget looks hovered. Disabled widgets ignore the pointer, whereas the browser still matches
:hover on disabled `.choice` buttons: a known small difference. No transitions, cursor shapes or keyboard yet.

Build plan step 10 is done: `mere-ui` has `fold`/`fold_with`, `tab` (strip and choice looks), `detail_tab`, `strip_gap`, `number`
(no typing yet) and the styles of the strip, choice and details rows; text looks have a letter spacing, paint order follows
CSS (positioned boxes last) and `mix` is public. No slider was built: neither application has one. `demos/widgets
--controls` shows them live and `docs/shots/controls-dark.png` and `controls-light.png` are the sheet (12 new `mere-ui`
tests with the page's numbers, 2 picture tests). Two observations read from the page's CSS, not seen in a browser:

- Hovering the pressed strip tab: `.strip button:hover:not(:disabled)` (specificity 0,3,1) beats `[aria-pressed=true]` (0,2,1),
  so the word turns `--text` on a `--text` ground and vanishes. `mere-ui` keeps the pressed word on hover on purpose.
- The comments at preview.html line 657 and launcher line 571 still say "slider", and `--track` is used by no rule.

Build plan step 11 is done: `mere-base` has `Event::Key`, `Event::Char`, `PointerDown.clicks`, `Key`, `Mods` and the `Clipboard`
trait; `mere-win32` turns `WM_KEYDOWN`, `WM_CHAR` (with surrogate pairs) and click counting into them and has `SystemClipboard`
(CF_UNICODETEXT; the manual round-trip test `cargo test -p mere-win32 --test clipboard -- --ignored` passed); `mere-ui` has the
editing model (`edit.rs`), `text_field` (Text and Compact looks) and the editable `number` with a `NumberSpec`, one focused
field with Tab order = build order and `Ui::want_redraw`. `demos/widgets --fields` is the live mode and
`docs/shots/fields-dark.png` and `fields-light.png` are the sheet (24 `mere-ui` field tests, 10 edit-model unit tests, 2
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

Build plan step 12 is done: `mere-ui` has `ellipsize` and `ellipsis`, a vertical scroll area (`scroll`, `scroll_style`:
thumb, 11 px gutter, bottom fade, wheel, thumb drag), hit tests cut to the visible part of a widget, `list_row`,
`list_head` and `trend_row`; `demos/widgets --lists` is the live mode and `docs/shots/lists-dark.png` and `lists-light.png`
are the sheet (11 `mere-ui` tests in `tests/list.rs`, 1 `Rect::intersect` test, 2 picture tests). Unverified, chosen by us
or read from the page, not seen in a browser:

- `GUTTER` 11 px (what the thin classic scrollbar reserves), `THUMB_MIN` 18 px and `WHEEL_STEP` 100 px per notch.
- Since the fluid-panels branch the bar has its own look (maintainer's request, not the page's): a 1 px track in the
  line colour, a 3 px thumb (muted, text colour on hover or drag) and a chevron arrow at each end that has content
  beyond it (`ARROW` 10 px of room at both ends of the track); the gutter stays 11 px.
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
`docs/shots/preview-1400.png`, `preview-1200.png`, `preview-1000.png` and `preview-1400-light.png`. `mere-ui` gained
`wrap_lines`, `paragraph`, `fill`, `select`, `TextLook::small` and `label`. Tests: 7 in `mere-ui/tests/shell_bits.rs`,
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
`map-relief-zoom8.png`). Built: `mere-base` has `Cmd::Image`, `Disc` and `Ring`; `mere-win32` draws them in Direct2D
(a premultiplied BGRA bitmap beside each D3D texture; nearest-neighbour); `mere-ui` has `canvas`, `Resp::pressed`,
`pointer` and `take_wheel`; `demos/preview` has `world.rs` (the out folder with flat land, structures and the cut),
`colour.rs` (the page's colours and a regex-free matcher), `paint.rs` (`paint()` for the five views, the choosing dimming,
the marks), `map.rs` (look, wheel, carry, grid, cross, ticks, points under the pointer), `app.rs` (`PreviewApp`),
`json.rs` (relief's copy; step 15 leaves one) and `tools/page_paint.mjs`. `paint()` is byte-identical to the page's own
code run by Node on `C:\Games\preview\out`, for 12 cases (five views x dark and light, a chosen-biome and a chosen-height
case); the biome and structure colours, the structure order and the flat count are equal too. Through the window path
the rest look shows each point as its texel, and at 8x zoom every block interior equals its texel exactly with 929
sharp boundaries (so Direct2D's nearest-neighbour is exact at a fractional scale). Tests added: 1 in `mere-base`, 4 canvas tests in `mere-ui`, 1 in `mere-win32`, in `demos/preview` 7 unit tests
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
  `mere-win32`. S2 asks for 30 notches, which the zoom range stops at its near end (`min_distance` = 40 steps).

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

Build plan step 17 is done: data and runs (`docs/shots/run-progress.png`; `export-done.png` was dropped before publication, as it showed a machine path). `preview` reads
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
learns. Stop is `taskkill /PID … /T /F` through `mere_base::Launcher` (new, no OS code; `mere_win32::SystemLauncher`
implements it with `std::process` and `CREATE_NO_WINDOW`, no new `windows` feature): it takes the generator's whole
tree, the game server included; the app does the same when it is dropped (serve.ps1's `finally`). Export writes
`biomes.png`, `flat.png` (not for a coarse map), `height.png`, `temperature.png` (each the whole map, whatever is
chosen: `paint()` encoded as is) and `section.png` (the last uploaded sheet, alpha kept) into
`exports/<seed>_<UTC stamp>/`, then shows "Exported to <folder>" with "Open the folder" (Explorer through the launcher)
and "Close" under the map.

Tests: `tools/stub_run.ps1` stands for `preview.ps1` (same parameters, same deleted files, `progress.json` written
atomically, `timing.json`, `run.json`, the same throw when killed; modes `finish`, `hold` and `fail` by
`$env:MERE_STUB`; it refuses any Out or Dir outside `target/tmp` before it creates anything and downloads nothing).
`tests/run.rs` R1 to R7 drive the whole application against it offscreen (R1: the 40 % picture
`target/tmp/run_40.png` and the stop that ends the stub's grandchild process; R2: the finished run becomes the map and
the pace learns 1.5 s of work; R3 failed, R4 refused, R5 queued, R6 dropped while running, R7 the guard), `tests/export.rs`
E1 to E4 (E1: the four PNGs equal `paint()` byte for byte and 200 stage pixels of `target/tmp/export_done.png` each equal
a texel of `biomes.png` around their place; E2 a chosen biome is not exported; E3 `section.png` is the sheet; E4 the
notice's two links, through a recording launcher), `mere-win32/tests/process.rs` X1 (stop takes the grandchild),
`mere-ui/tests/run_bits.rs` N1 to N3, 10 unit tests in `run.rs` and `text.rs` (the command line, seeds, the pace, the
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

- `mere-base`: `Cmd::Polyline` (an open stroke, round joins, flat ends) and `Event::Scheme { dark }`. `mere-win32`: the
  polyline as a Direct2D path geometry with one stroke style (`D2D1_LINE_JOIN_ROUND`, flat caps), `system_dark()` (the
  registry's `AppsUseLightTheme`; missing or an error is light, as Chrome reads it; the only new `windows` feature is
  `Win32_System_Registry`) and `WM_SETTINGCHANGE` ("ImmersiveColorSet") turned into `Event::Scheme` when the value differs
  from the last one sent. `mere-ui`: `hover_area`, `list_part` (the `.part` heads), round mark swatches (`RowOpts.mark`),
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
- Choices where the brief was silent: the first two commits of the brief became one (mere-base alone would not compile
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
inside `mere-win32`), and looked at `figures-1400`, `history`, `stage-3d-relief`, `section-1400` and `run-progress`.
The step 15 and 17 shots still show step 13's placeholder figures in the right column: they are those steps' record.

Step 19 (2026-11-15; its fixed panels, its 1000x600 minimum and its single bar form are superseded by the fluid panels section further down) answered the maintainer's remark that the page had too many breakpoints: the world preview's
panels never move (left 320, right 330, gap 28, padding 18/28/22/28; step 19.5: 10/28/22/28), the centre is one column that scales by one
rule (the map's square side is `min(W - 762, H - 144)` (step 19.5: `H - 136`); the brief's `W - 752` left ten pixels out: 320 + 330 + 2 x 28 +
2 x 28 = 762), and the top bar has one form from 1000 px up. The window has a minimum of 1000x600, so no layout
below it exists. The bar is the launcher's, on a frame of our own (`WM_NCCALCSIZE` returns 0, `WM_NCHITTEST` asks
`App::hit`, the buttons are drawn by `mere-ui`'s `chrome.rs`), with the tool switch Home / preview. The same step
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
- PLACEHOLDER: `HOME_LATER` (at step 19 it said Home came with step 21; since the final review it says Home belongs to
  the launcher) and the Home pane it fills, which is empty otherwise; Home's own content belongs to step 21.
- New `windows` features, all inside `mere-win32`: `Win32_Graphics_Dwm` (`DwmExtendFrameIntoClientArea`, to keep the
  window's shadow on the own frame) and `Win32_UI_Controls` (`TrackMouseEvent`, so that the pointer leaving the window
  is known). `RegisterOcclusionStatusWindow` needed nothing new (`Win32_Graphics_Dxgi` was already on).
- Report for the maintainer: in the pack's repository, `git status` shows ` M pack/pack.toml` with an empty content diff;
  only its modification time (2026-11-15 00:11) differs. Nothing in this work touched it (that repository is read
  only here); the likely cause is a launcher (webview) run in that checkout, since ignored files under `run/app/webview/EBWebView`
  were also written between 00:11 and 00:18. The maintainer should confirm; `git update-index --refresh` there would
  clear it, and was not run.
- Choices where the brief was silent: `Cmd::Shape` (a path with the winding rule) was added to `mere-base` for the
  icons, drawn crisp with its vertices on device pixels; the icon grammar is the reduced SVG one (`M m L l H h V v Z z`);
  a hidden window's `tick()` looks at the run and leaves the end for the next frame, which holds the device;
  `export.json` writes `ores.added` as null (no editor for the added rows natively) and `ores` as null outside the
  section view; the Home pane is a placeholder text; `UNREADABLE` is our own sentence in the page's style.
- UNVERIFIED, moot since step 19.5 (the hairline is gone): the bar's hairline at scale 1.5 spills one device row below the bar and was not compared with Chrome;
  no live check of the window (the cursor, the tooltip delay, the minimum size, occlusion, a DPI change between monitors,
  Aero snap on the own frame) was made, only the pure functions behind them and the offscreen pictures.
- Also UNVERIFIED: the DWM margins of 1 px on a live window (a system-coloured line at the top would mean using 0),
  the maximise inset, `WM_NCMOUSELEAVE`, OrbitControls' cursor, the tooltip's look (ours, in the page's tokens), and the
  `ores.added` default.
- Not built: glyph tabs and `condense`, the options transition, the marker blink, snap layouts on Maximise, the close
  confirmation, `#account`, caret blink, focus rings, changed marks and their hints; the 3D and structures pictures in
  the export; the window's dark title bar (the own frame has none); IME.

Step 19.5 (2026-11-15) answered the maintainer's remark on `bar-1400.png`: the header should float, with no line
separating it, and the gap under the bar was too big. The bar's hairline is removed (a deliberate departure from the
launcher page's `#titlebar` border-bottom; the pack's repository is not changed). The content now starts `CONTENT_TOP` =
10 px under the bar, for the preview and for Home, instead of the page's 18. The rule is now
`side = min(W - 762, H - 136)` (40 + 10 + 22 + 64 = 136); the panels start at y 50 and are `H - 72` high. The bar's
1 px bottom padding is kept so no part moved. Home's pane is inset like the preview's page and holds a `homeBody`
box (step 21 builds in it).
- Evidence: `cargo test --offline --workspace` 380 passed, 0 failed, 1 ignored; B6 asserts no full-width line in the top
  60 px and the page's ground in rows 39..50 at scales 1, 1.5, 2. Regenerated in `docs/shots`: `bar-1400`,
  `bar-1400-light`, `bar-x2`, `bar-home`, `bar-maximized`, `bar-hover-close`, `responsive-1920`,
  `responsive-1400x600`, `responsive-section-1000`. The other shots are earlier steps' records and were not redone.
- PLACEHOLDER: the Home pane is unchanged (`HOME_LATER`, now inside `home_body`).
- Not built: unchanged (no animation, `#account`).

Step 20 (2026-11-15) built the input method (IME, IMM32 only, no TSF) in the text field.
- Built: `mere-base` has `Event::Ime(Ime)` (`Start`, `Update { text, cursor }`, `Commit { text }`, `End`, one per IMM
  message; `Event` is no longer `Copy`) and `App::ime_spot()` (the caret in dip, None turns the input method off).
  `mere-ui` counts cells: a wide char (East Asian Wide or Fullwidth, a reduced UAX #11 table in `text.rs`) takes two
  cells inside a field, for the text's width, the caret, the selection, the pointer and the scroll; `max_chars` still
  counts chars. The composition (preedit) is shown inline at the caret, with one underline under all of it; the value
  the application holds changes only at the commit (a departure from the browser's input events, on purpose), which goes
  through the edit model as one undo step and replaces a selection. A cancel (an empty update, then the end) leaves the
  text and the selection as they were. Keys and chars are dropped while a composition shows; a blur drops it (the system
  is told to cancel; Chrome commits on blur, which would here commit after the focus moved, possibly into another
  field). Number fields ignore the input method. `mere-win32` (`ime.rs`, the only module that names IMM): the
  composition becomes `Event::Ime`, the system's own composition window is hidden (`WM_IME_SETCONTEXT`), the candidate
  window is placed under the composition's start with `CFS_EXCLUDE`, `WM_IME_COMPOSITION` never reaches
  `DefWindowProcW` (no second typing through `WM_IME_CHAR`), and no input context is associated with the window unless a
  text field has the keyboard. The new `windows` feature is `Win32_UI_Input_Ime`; `Cargo.lock` did not change.
  `demos/widgets` has the input method sheet (`ime_sheet`, `docs/shots/ime-dark.png`, `ime-light.png`); both apps pass
  `Ui::ime_spot` to the backend.
- Evidence: `cargo test --offline --workspace` 402 passed, 0 failed, 1 ignored, no warning (22 new: 3 wide-cell tests in
  `mere-ui/tests/field.rs`, 9 in `mere-ui/tests/ime.rs`, 6 unit tests of `mere-win32/src/ime.rs`, the `key_of(0xE5)` test,
  2 pixel tests in `demos/widgets/tests/ime.rs`, 1 in `demos/preview/tests/ime.rs`). No other shot was regenerated.
- PLACEHOLDER: none.
- UNVERIFIED: the live input-method path in its entirety (no Japanese or Chinese input method is installed or driven
  here); the candidate window's place on screen; the glyph centring in two cells, which assumes the fallback font
  advances 1 em (IBM Plex Mono has no CJK glyphs); the behaviour at a display scale of 150%.
- Not built: the composition's attributes and clauses (one underline only, no target-clause look), reconversion
  (`IMR_RECONVERTSTRING`), TSF, wide cells outside fields (labels, lists and the ellipsis count one cell per char),
  half-width katakana drawn at its own advance inside its one cell, and commit on blur (we cancel).
- Choices where the brief was silent: `byte_near_cell` and `byte_under_cell` take the pointer's position in cells (a
  float); a message of `WM_IME_COMPOSITION` that carries neither the result nor the composition string (only attributes
  or the caret) sends nothing, and only lparam 0 sends an empty update; the spot is clamped to the field's content box;
  `byte_at_col` stays, for tests only.
- The manual check, by the maintainer:
  1. Add Japanese (Microsoft IME) under Settings > Time & language > Language & region.
  2. `cargo run --offline -p widgets -- --fields`, click the seed field, switch to Japanese with Win+Space and then
     Alt+`, and type "nihongo". It must show にほんご underlined inline at the caret, with no separate floating
     composition box.
  3. Space: the candidate list opens just under the field, at the start of the composition, not at the top-left of the
     screen.
  4. Enter: 日本語 appears once, not twice, and the caret is after it.
  5. Type "ka" and press Escape (twice if needed): the composition disappears and the text is unchanged.
  6. Click the "Vertical scale" number field and type with the input method selected: no composition, the
     input-method indicator shows it off, and digits go in.
  7. Click elsewhere during a composition: it is dropped.
  8. Repeat 2-5 in `cargo run --offline -p preview` on the seed field, at 100% and at 150% display scale.
  9. Optionally, Microsoft Pinyin: type "zhongwen", Space, then 1.
  10. Logs at Debug show "ime on" / "ime off" and the commit counts.

### Step 21: the native launcher (2026-11-15)

- Built, in this repository (branch `feat/native-launcher`):
  - `mere-ui`: the launcher's icon looks (`IconLook::Square`, `Filled`, `Bare`, joined squares), `hover_box`,
    disabled text and number fields (`FieldOpts`, `number_with`), `ButtonLook::Underlined`, `toggle_with`, `Ui::clock`,
    the select with its menu (`menu.rs`, `select_menu`: 25 px box, 20 px rows, painted last and outside every clip,
    Escape closes it), and `anim.rs` (`ease` = CSS `cubic-bezier(.25,.1,.25,1)`, `Slide`, 220 ms).
  - `mere-win32`: `reduced_motion()` (`SPI_GETCLIENTAREAANIMATION`, read once by the app).
  - `demos/widgets`: the launcher controls sheet (`--launcher`, `launcher_controls_<theme>.png`).
  - `demos/preview`: `home.rs` (the `Home` trait: build, act, poll, wake_in, shown, account, as_any; hosted by
    `PreviewApp::with_home`, drawn by `picture_with_home`), the bar's account (`#account`, ellipsis beside the pack's
    name) and the 220 ms slide of the preview tool's views (the options box clips at width = value x natural while it
    moves, Auto when settled or when reduced motion is on), and `Runs::server_dir`, `out_dir`, `exports_dir`.
  - In the pack's repository (branch `feat/native-launcher`): `launcher/app` is now a library (`api`, `system`) plus a
    binary behind the `window` feature (so `launcher/native` pulls no tao or wry); `launcher/native` is the new
    package (`LauncherHome`, a bridge to `launcher::api::Api` in process with origin `native://launcher`, a fake api for
    tests and pictures, `main.rs` with the window and `--shot`); `scripts/launcher.ps1 -Native`; docs.
  - The Home has no switch: two columns that fill the window continuously. The page's 860 one-column form was built
    first, but the window never goes under 1000 (each part is still 448 px wide there), so it could not be reached and
    the final review removed it (`HOME_SWITCH`, `HomeIds::stacked`, the 800 and 860 pictures).
  - Page rules dropped on purpose: the 1120 cap, the padding change at 860, and `@media (max-width: 640px)` hiding the
    account.
- PLACEHOLDER: `HOME_LATER` stays only in the standalone `preview` binary's Home. The launcher's Home is real.
- UNVERIFIED:
  - the live window as a whole (nothing was opened by an agent);
  - the select popup's look and the 25 px height of the select against Chrome;
  - the timing of the reversed slide (a slide reversed half way);
  - `SPI_GETCLIENTAREAANIMATION` as the equivalent of Chrome's `prefers-reduced-motion`;
  - the startup delay of the two `pwsh` calls and of the world's load before the window shows;
  - the real api through the native interface (tests use the fake; `api_smoke` only builds the `Api`);
  - a real preview run from the launcher (it writes into `C:\Games\preview`; no agent ran it);
  - a display scale of 150%;
  - the stand-in icon shapes in the widgets sheet (they are not the launcher's real icons).
- Not built:
  - `condense()` (the views turning into icons when the bar is narrow): replaced on 2026-11-15 by the bar's overflow, see "Bar overflow" below;
  - the marker's blink;
  - the confirmation on closing while a run computes;
  - the window's remembered place and size;
  - the taskbar's cube icon;
  - focus rings;
  - the Ctrl+wheel guard and the context menu;
  - keyboard in the select's menu beyond Escape;
  - a multi-line text area (the Java field is one line in a 52 px box);
  - the page's "empty number = min";
  - a live change of reduced motion (read once at start);
  - bringing a running native window to the front (the native window writes no `hwnd.txt`; a refused second start is
    logged, and a stale `hwnd.txt` is removed once the lock is held);
  - the page's `@media 640`, the 1120 cap, the 860 padding change.
- Choices where the brief was silent:
  - the bar normalises a state that names a tool without its slide (Home with the views open becomes closed, and the
    reverse); `options_w` lives in `PreviewState`; `ShellIds.options` was added;
  - the select's width is the parent's (`Pct(100)`) and its menu is `20 n + 2` tall;
  - icon tests read draw-list colours (mere-ui has no renderer);
  - disabled number fields show 0 when the server has no properties;
  - Home's first poll asks both endpoints whatever the tool;
  - `Rig` in `demos/preview/tests/bar.rs` sets `reduce_motion` true, so the older tests do not animate;
  - `launcher/native` keeps its own `Cargo.lock` (copied from `launcher/app`); its toolkit paths, written in one place,
    name this repository's main checkout by an absolute path since the merge of 0.2.0, and become
    `../../../mere-ui` once the pack's repository merges its branch;
  - `main.rs` opens 1280x880 raised to the least size; `--shot` defaults to the dark theme;
  - at 1400x600 the two parts never overflow with the preview's 10 px top (they end at 582 of 600), so the test and the
    picture of the overflow use a long server note (a 412 EULA error on start); a deviation from the brief;
  - "PlayerOne" is ellipsised in the preview tool at 1000 (179 px left of the strip), so the account tests check a
    prefix and that the right edge stays left of the strip;
  - `FIXED_NOW_MS` is 1_795_262_400_000 as the brief says; it is 2026-11-21 12:00 UTC, which only matters in
    differences.
- Evidence: `cargo test --offline --workspace` 438 passed, 0 failed, no warning (36 new, 402 before). In
  `outpost-tools/native`: 37 tests (6 lib, 1 `api_smoke`, 14 `home`, 10 `layout`, 6 `pictures`; the first
  picture test checks that the native frame is byte-equal to `picture_with_home`); `launcher/app` builds with
  `--locked` and has 2 lib tests; `cargo build --release` of `launcher/native` passes (not run). Pictures in
  `docs/shots`: `launcher-controls-dark.png`, `launcher-controls-light.png`, `bar-slide-mid.png`,
  `launcher-home-{1000,1400,1920,1400x600,light,launch}.png` (`launcher-home-settings.png` was dropped before publication, as it showed a machine path),
  `launcher-preview-{1000,1400,1920,light}.png`. There is no 800 picture: the window holds 1000 at the least, so the
  sweep of the launcher's Home is 1000, 1400, 1920 and a short height (1400x600), the layout tests run every width
  from 1000 to 1920, and an 800 request is shown to become 1000 (`l9`).
- The manual check, by the maintainer (the same twelve points are in the pack's setup notes, "Native launcher"):
  1. Close the page launcher. In the pack's repository: `$env:WORLD_PREVIEW_DIR = '..\world-preview'; .\scripts\launcher.ps1 -Native`.
  2. Home: the profiles with their lines, the account after MINECRAFT; a row lights under the pointer.
  3. Resize from the narrowest to full screen: the two parts widen smoothly, nothing jumps or is cut.
  4. A profile's Settings: Opens on > a server, type an address, Enter: "Saved."; Prism's instance shows it; set it back.
  5. Memory above the maximum, Enter: clamped. Game folder > Open: Explorer.
  6. Play a profile: Starting..., Syncing the pack..., Loading the game... N s, Running.
  7. Dev server: the settings icon unfolds the fields; Start opens its console; Running, localhost:port; Stop: Saving and stopping..., then stopped.
  8. World preview: the views slide open in about 0.2 s, Home slides them shut; with Animation effects off and the launcher restarted, they switch at once.
  9. Generate: the real `preview.ps1` runs into `C:\Games\preview`; the tool says computing, then ready. Export writes under `C:\Games\exports\world-preview`.
  10. Switch Windows to light: the window follows.
  11. Close: the window and a run under way stop; `run\launcher.log` has "started (native)" and "stopped: the window was closed".
  12. `.\scripts\launcher.ps1` with no switch still opens the page launcher.

Final review of step 21 (2026-11-15): eleven findings, all real, all fixed. In the pack's repository: Play no longer reads
the launch state before its POST answered (`next_launch` waits at infinity until the Play answer, as the page awaits
the POST), which removes a race that could end the follow early; a fast click on Back now saves first (saves are
sorted before navigation in `act`); a refused second start is logged and a stale `hwnd.txt` is removed; Home's
unreachable stacked form and its 800/860 pictures are gone. Here: a minimised window now ends its run and tells it at
once (`tick` runs `finish` without a device; only the mesh upload waits in `upload_pending`); labels measure, cut and
wrap by cells as fields do (wide glyphs); the standalone Home placeholder no longer names a step; no probe line before a
map exists (as the page); the 1100 px list variant (`narrow`) is removed; crisp icon edges of equal width stay equal
(origin snapped first, offsets rounded by one rule; the maximise box now has four edges of the same width, checked
at four scales and five fractional positions); the 800 pictures and this list's claim about them are corrected.
Not settled by tests: the Play race and the Back-after-save order are fixed by construction in the real window only
(inline bridges answer at once), so they are UNVERIFIED live; the save POST and the list reload that Back sends right
after it go out on separate threads and their order is not guaranteed. The wide-glyph measure assumes the fallback font
draws a wide glyph two advances wide, UNVERIFIED in a real window.

Closing state of the chapter: the toolkit, the native world preview (steps 13 to 20) and the native launcher (step 21)
are built and covered by tests and pictures; all that is left is manual for the maintainer, the twelve points above
plus: Play a profile twice in one session (the second must follow its own start), edit Java arguments and click the
back arrow at once, minimise during a Generate and look at the taskbar title, a profile named in Japanese. Still
UNVERIFIED: the live window, a display scale of 150%, the real `preview.ps1` run from the launcher. Not built: the list
above. The build plan has no further step.

Next: the maintainer's manual check of the native launcher (above); then the orchestrator merges `feat/native-launcher` of both repositories and repoints `launcher/native`'s paths at `../../../mere-ui`. The build plan has no step after 21.

Provisional choices, made by the session on 2026-11-14 and not settled: the maintainer had no view of their
own and asked to see the result first, so revisit both once the panels can be looked at.

- No height key over the 3D view: applied in step 15. Nothing is drawn over the 3D in the preview (no key, no plate, no facts); the height list stays the right column's list, which does not follow the view yet.
- The overlay's facts belong to the side panel, which is the right column: applied in step 15 for the preview. The relief demo, a standalone window with no side panel, keeps its filled plate, because grey text without a ground is unreadable over bright snow.

### Bar overflow (2026-11-15)

The maintainer chose "form A" for a narrow bar. The tools show their names ("Home", "Preview", catalog `TOOL_NAMES`; the
longer `MODES` stay their tooltips), and `overflow.rs` works out, before the bar is built, what to give up. The order is
the account (an ellipsis down to a 3 letter stub, then gone whole), the greyed views into "More" (`Ui::tab_menu`, on the
`select_menu` list, with disabled rows), the word MINECRAFT, then the other views into "More" one by one from the strip's
end; the pressed view never goes. Widths are arithmetic from the monospaced font (`TextLook::width`) and the styles'
paddings; `tests/bar_overflow.rs` checks the predicted strip width against the laid out one.

Decisions the brief left open: (1) the first state that fits is taken among cumulative steps, and a step that buys no room
is not kept. The brand (cube, word) is 109 px, narrower than the 138 px of the window's buttons it shares a side with, so
dropping the word never frees anything: it stays in every real window, and the rule is there for a narrower pair
(tested with smaller buttons). (2) The views' arrangement is that of the preview tool open, whatever tool is on, so the
views do not shuffle while they slide; the account and the word follow the tool on show, so Home keeps them at 1000.
(3) The marker stays on the icon's top right corner.

At 1000 px with the loaded world (stopped at a coarse detail, so Flat, Structures and Section are greyed): the account is
gone, "More" holds Flat, Structures and Section, Biomes, Relief and Temperature stay, the word stays. With every view on:
"More" holds Structures and Section (the account gone as well). With the current widths and an account such as "Louis", nothing is given up from 1035 px up; the account goes from 1034 down,
the greyed views move into "More" from 1014 down, and with every view on Section moves from 1014 down, then Structures from 1008 (the
words in the tables are measured, so another account name or catalog word moves these numbers by itself).

## Goal of this run

Replace the world preview's three.js view (`..\world-preview\page\preview.html`, `refresh3d()` near
line 1842) with a native Rust program, through a small UI kit of our own. Done for this run means: the
`relief` demo draws the real terrain from `C:\Games\preview\out` in 3D, in a Win32 window and as a `--shot` PNG,
and the PNG has been looked at and compared with the web preview's look.

## Decisions already taken (do not reopen)

- Direct3D 11 + Win32 directly. No tao, no wry, no web page, no Slint, Electron or Tauri.
- Only `mere-win32` names Windows; the other crates are portable so a Linux backend can replace it later.
- No third-party crate except `windows = "=0.62.2"` (already in the cargo cache), and only in `mere-win32`.
  Open-source libraries (Taffy, microui, egui, Clay) are read for how they do things, never depended on.
- PNG, lossless, our own encoder (`mere-png`, done): hairlines and text must not blur, so no JPEG or WebP.
- Reversed-Z infinite perspective (clear depth 0, compare GREATER) instead of three.js's logarithmic depth.
- Consolas first; IBM Plex Mono (embedded woff2 in preview.html, SIL OFL) comes after step 6.
- Work happens on `main` in this young repository, which has no remote: nothing to push.
- Commits under the maintainer's name only: no `Co-Authored-By`, no "Generated with" line.

## Where it stands

- Done and committed (`692a87a`): `mere-base` (Color, Vertex, Material, the `Gfx` and `App` traits, Event,
  math with column-major `Mat4`, `perspective_reversed`, `look_at`, the `Orbit` camera, `DrawList`, the log
  hub with `observe!`) and `mere-png` (encoder with filter heuristic and fixed-Huffman deflate, plus a decoder
  of our own files for tests). 8 tests green.
- Drafted, never compiled, untracked: `crates/mere-win32/` with `Cargo.toml`, `src/lib.rs` (`shot()`) and
  `src/renderer.rs` (device with WARP fallback, swap chain or offscreen texture, D2D on the DXGI surface,
  DirectWrite text, shaders compiled at start with `D3DCompile`, STAGING readback). `src/window.rs` (the
  `run` function it exports) does not exist yet. Expect compile errors against windows-rs 0.62.2 signatures.
- The workspace `members` in the root `Cargo.toml` list only `mere-base` and `mere-png`; add `mere-win32` and
  `demos/relief` as each one comes to exist.

## Steps left

1. **Offscreen renderer** (`mere-win32`): compile `renderer.rs`; a test or tiny example clears to a colour,
   draws one flat triangle and a `Frame`/`Text` command, writes a PNG. Pass: the PNG shows them.
2. **Window** (`window.rs`): `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`, a Win32 class and window,
   events mapped to `mere_base::Event` (size, pointer, wheel, close, `WM_DPICHANGED`), `GetMessageW` when idle
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

Data for tests: a world preview out folder named by `PREVIEW_DATA_DIR` (the one used here has seed 94427382). Without it, the tests that need it say "skipped".

## Workflow shape

The steps depend on each other, so the workflow is a chain, not a fan-out; keep it under 10 agents.

- Each step: one implementing agent (Sonnet) that writes the code, runs `cargo build` and `cargo test` in
  `mere-ui`, and commits once it compiles (small commits, `feat:`/`fix:` messages in
  English); then one checking agent (Sonnet) that rebuilds, runs the step's pass check, opens the PNG and
  answers pass/fail with reasons. On fail, the implementer gets the reasons back, at most two more rounds.
- The orchestrator (the session) reads each verdict, looks at the final PNGs itself, and reports in French.
- Every agent prompt carries the rules of `AGENTS.md` and the commit rule above.

## Fluid side panels (branch feat/fluid-panels, 2026-11-15)

Replaces step 19's fixed panels (left 320, right 330, minimum window 1000x600). Decided with the maintainer; the prototype is a page outside this repository.
- The numbers set by hand are in `demos/preview/src/panels.rs`: panel minimum 220, natural cap 330, map minimum 238, gaps 28 (beside) and 16 (stacked), height 600. The stacking width (790 with both panels open), the least window (`MIN_SIZE`, const arithmetic: the larger of the narrow bar, `overflow::NARROW_BAR`, and 2 x 220 + 3 x 16) and the bar's centred threshold are derived.
- Natural width = widest row on one line (label, field, gaps, scrollbar gutter) under the cap: 319 settings, 321 figures. The rows are the panels' fixed words, never a run's data. Labels wider than their cell wrap.
- Beside: the map's column shrinks first, then the panels toward 220 in proportion to their slack. Grips (8 px, in the gap) drag, no maximum, double click resets; the width lives in `PreviewState::panel_w`, not on disk. Stacked: the stage on the half of the height under the bar, the panels below in equal columns, each one scroll area (no nesting), no rule.
- The bar in the preview tool: the settings panel's button replaces the brand, the figures panel's sits before the window's buttons; below the width where the tools can be centred they sit left, then 2D and 3D go into More (`overflow.rs`). Home keeps brand and account.
- Pictures: `docs/shots/responsive-*` and `bar-*`. `responsive-section-1000.png` is `--data C:\Games\preview\out-section --view section`.
- Checks on this machine: the loaded `C:\Games\preview\out` is a coarse 320-sample world without a section, so the section, map, figures, page_* and run tests that need it already failed before this branch (same tests, same reasons).

## The preview and the relief demo leave (2026-11-17)

- demos/preview (package `preview`) and demos/relief left this repository: the world preview's window and the relief demo now live in the outpost-tools repository (https://github.com/louisfifre/outpost-tools, folders `preview/` and `relief/`), which depends on this kit by git tag. The text above is the history of how they were built, and its paths under `demos/preview` and `demos/relief` name files that are no longer here.
