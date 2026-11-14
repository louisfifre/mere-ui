# Handoff: steps 0 to 6, to run as a workflow

Written 2026-11-14 at the end of the session that designed the kit. The next session starts here and launches
a Workflow (the maintainer asked for one in words: "le mieux, ce serait de lancer un workflow", so the opt-in
is explicit). Read `AGENTS.md` and `docs/design.md` first; this file says where the work stands and how to
finish steps 0 to 6.

## Status: steps 0 to 6 done (2026-11-14)

The build run finished. `relief <data-dir>` draws `C:\Games\preview\out` in 3D in a window and as `--shot`; the
shots in `docs/shots` were compared with a capture of the page's 3D view (`world-preview/scripts/capture.ps1`,
`/?view=height,relief3d`) and match in framing, colours, water and edges. A review pass then added 4x MSAA
(the page asks WebGL for antialias), hairlines of one logical pixel, the page's biome total and wording,
OrbitControls speeds, a camera kept above the ground within a zoom range, and robustness fixes in the window
(no `&mut` aliasing across re-entrant calls, a cleanup guard, safe capture, device-loss rebuild).

Not exercised at runtime: device loss, capture lost mid-drag, a `WM_DPICHANGED` between monitors, and a
look at the live window's pixels (only its start, answer and clean exit were checked). Open questions for the
maintainer: a height key in the native view (the page keeps it in its side panel), and the overlay's filled
plate against a hairline-only frame like the page's panels. Next: IBM Plex Mono, then the page's panels.

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

