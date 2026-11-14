# native-ui design dossier (rust-native-ui, Windows first)

Status of claims: ESTABLISHED = read in a source or official doc (URL given). INTERPRETATION = my reading. UNVERIFIED = not checked. Line counts are my estimates, not measured.

Key idea that removes work: styles are built in Rust, not parsed from CSS. So `calc()`, `min()`, `var()`, `clamp()` and container queries disappear: they become ordinary Rust arithmetic and `if width < 1100` in the function that builds the tree. The toolkit only needs the layout *model*, not a CSS parser.

---------------------------------------------------------------------------
## 1. Crate / module tree and public API

Repo `rust-native-ui`, one Cargo workspace. Only external dependency: `windows` (bindings), and only inside `nui-win32`. All other crates have an empty `[dependencies]`.

```
rust-native-ui/
  nui-base/     geometry, colour, input events, platform traits. No OS code.   (~400 lines)
  nui-png/      PNG encoder (stored deflate) + optional diff tool               (~120)
  nui-layout/   style structs, tree, flex + grid + absolute, measure callback  (~900)
  nui-ui/       Ui context, ids, draw list, widgets, theme, text field model    (~2200)
  nui-win32/    Win32 window + D3D11 + D2D/DirectWrite implementing the traits (~1800)
  nui-shot/     headless harness: render a View offscreen to PNG (uses a backend) (~150)
  examples/     gallery (every widget), used for screenshots
```
Later, a Linux backend is a sibling crate `nui-x11` (or wayland) implementing the same traits. `nui-ui` and above never name `windows`.

### 1.1 nui-base (platform contract)

```rust
#[derive(Copy, Clone, Default, PartialEq)] pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
#[derive(Copy, Clone, Default, PartialEq)] pub struct Vec2 { pub x: f32, pub y: f32 }
#[derive(Copy, Clone, PartialEq)] pub struct Color(pub f32, pub f32, pub f32, pub f32); // linear-free sRGB, premultiplied at draw time
impl Color { pub const fn hex(rgb: u32) -> Color; pub fn with_alpha(self, a: f32) -> Color; }

pub enum Key { Left, Right, Up, Down, Home, End, Backspace, Delete, Enter, Escape, Tab, A, C, V, X, Z, /* ... */ }
pub struct Mods { pub shift: bool, pub ctrl: bool, pub alt: bool }
pub enum Cursor { Arrow, IBeam, ResizeH, ResizeV, Hand }

pub enum Event {
    Resized { w_px: u32, h_px: u32, scale: f32 },
    PointerMoved(Vec2), PointerDown { pos: Vec2, button: u8, clicks: u8 }, PointerUp { pos: Vec2, button: u8 },
    Wheel { pos: Vec2, dy: f32 }, KeyDown { key: Key, mods: Mods }, Char(char),
    Ime { preedit: String, commit: Option<String> },   // phase 2
    Timer, Wake, CloseRequested,
}

/// Everything the OS gives us. One implementation per platform.
pub trait Window {
    fn size_px(&self) -> (u32, u32);
    fn scale(&self) -> f32;                       // 1.0 = 96 dpi
    fn request_redraw(&self);                     // on-demand: the loop sleeps until an event or this
    fn request_timer(&self, ms: u32);             // caret blink, progress poll
    fn set_cursor(&self, c: Cursor);
    fn clipboard_get(&self) -> Option<String>;
    fn clipboard_set(&self, s: &str);
    fn wake_handle(&self) -> Waker;               // Send+Clone, lets a worker thread wake the loop
}

/// Text measurement and drawing. Monospace only, so metrics are simple.
pub trait Text {
    fn cell(&self, size: f32) -> (f32, f32);                         // advance width, line height
    fn ascent(&self, size: f32) -> f32;                              // for baseline alignment
    fn measure(&mut self, s: &str, size: f32, wrap_w: Option<f32>) -> Vec2;
    fn caret_x(&mut self, s: &str, size: f32, byte: usize) -> f32;
    fn hit(&mut self, s: &str, size: f32, x: f32) -> usize;         // byte index at x
}

pub enum Cmd {
    Rect { r: Rect, fill: Color },
    Stroke { r: Rect, w: f32, c: Color },                           // hairline = 1 physical px
    Line { a: Vec2, b: Vec2, w: f32, c: Color },
    Text { at: Vec2, s: String, size: f32, c: Color },
    Image { id: ImageId, dst: Rect, src: Rect, nearest: bool },     // the 2D map
    PushClip(Rect), PopClip,
    Viewport { id: u32, r: Rect },                                  // hole the app fills with D3D
}
pub struct DrawList { pub cmds: Vec<Cmd> }

pub trait Gfx {
    fn begin(&mut self, clear: Color) -> Result<(), GfxError>;
    fn ui(&mut self, list: &DrawList, text: &mut dyn Text);          // draws cmds in order up to each Viewport
    fn upload_image(&mut self, id: ImageId, w: u32, h: u32, rgba: &[u8]);
    fn mesh(&mut self, id: MeshId, verts: &[Vertex], idx: &[u32]);
    fn draw_mesh(&mut self, vp: Rect, mesh: MeshId, cam: &Camera, tex: Option<ImageId>, blend: Blend);
    fn end(&mut self) -> Result<(), GfxError>;                       // Present (or no-op offscreen)
    fn read_pixels(&mut self) -> (u32, u32, Vec<u8>);                // RGBA8, for PNG
}

pub trait Backend {
    /// Runs the loop until close. `frame` is called only after an event/redraw request.
    fn run(self, app: &mut dyn App) -> !;
    fn offscreen(w: u32, h: u32, scale: f32) -> Box<dyn Offscreen>;   // headless Gfx + Text, for nui-shot
}
pub trait App {
    fn event(&mut self, win: &dyn Window, e: Event);
    fn frame(&mut self, win: &dyn Window, gfx: &mut dyn Gfx, text: &mut dyn Text);
}
```
Design choices (INTERPRETATION): the draw list is plain data, so the same list can be drawn by D2D, by a Linux backend, or dumped as text for tests. `Cmd::Viewport` lets 3D sit between UI layers: the backend ends D2D, draws the mesh with a D3D viewport+scissor, then resumes D2D.

### 1.2 nui-layout

```rust
pub enum Dim { Auto, Px(f32), Pct(f32) }                  // ch/em/cq are converted to Px by the caller
pub enum Display { Flex, Grid, Block, None }
pub enum Track { Px(f32), Auto, MinMaxFr0(f32) }          // MinMaxFr0 == minmax(0, Nfr); bare `1fr` comes with step 21
pub struct Tracks { /* up to 8 tracks inline, so that Style stays Copy */ }   // Tracks::new(&[..]), as_slice()
pub enum Position { Static, Relative, Absolute }          // Relative marks a containing block, no offset; no Fixed yet
pub struct Insets { pub left: Dim, pub top: Dim, pub right: Dim, pub bottom: Dim }   // Insets::all(Dim::Px(0.0)) == inset: 0
pub enum Overflow { Visible, Hidden, ScrollY }            // ScrollY also clips x; one field, not overflow_x / overflow_y
pub struct Style {                                        // Copy
    pub display: Display, pub column: bool, pub wrap: bool,
    pub gap: Vec2,                                          // (col gap, row gap)
    pub grow: f32, pub shrink: f32, pub basis: Dim, pub order: i32,
    pub w: Dim, pub h: Dim, pub min_w: Dim, pub min_h: Dim, pub max_w: Dim, pub max_h: Dim,
    pub aspect: Option<f32>,
    pub pad: Edges, pub margin: Margin /* each side Px or Auto */,
    pub align_items: Align, pub align_self: Option<Align>, pub justify: Justify,
    pub cols: Tracks, pub col_span_all: bool,               // grid-template-columns; grid-column: 1/-1
    pub justify_items: Align, pub justify_self: Option<Align>,   // the other half of place-items
    pub position: Position, pub inset: Insets, pub overflow: Overflow,
}
pub enum Align { Start, Center, End, Stretch, Baseline }
pub struct Tree { /* arena: Vec<Node>, children: Vec<Vec<NodeId>> */ }
impl Tree {
    pub fn new_node(&mut self, s: Style) -> NodeId;
    pub fn append(&mut self, parent: NodeId, child: NodeId);
    pub fn set_leaf(&mut self, n: NodeId, m: Box<dyn FnMut(Avail) -> Measured>); // text / canvas
    pub fn compute(&mut self, root: NodeId, size: Vec2);
    pub fn rect(&self, n: NodeId) -> Rect;                  // relative to parent
    pub fn content_size(&self, n: NodeId) -> Vec2;          // for scrolling
}
pub struct Measured { pub size: Vec2, pub baseline: f32 }
```

### 1.3 nui-ui (immediate mode)

```rust
pub struct Ui { /* ids, hot/active/focus, scroll offsets, fold states, tree, draw list, theme */ }
impl Ui {
    pub fn begin_frame(&mut self, input: &Input, size: Vec2, text: &mut dyn Text);
    pub fn end_frame(&mut self) -> &DrawList;
    // containers: closures build the layout tree for this frame
    pub fn node(&mut self, id: Id, s: Style, children: impl FnOnce(&mut Ui)) -> Resp;
    pub fn label(&mut self, s: &str, tone: Tone);
    pub fn button(&mut self, id: Id, s: &str) -> bool;
    pub fn toggle(&mut self, id: Id, on: &mut bool, s: &str) -> bool;
    pub fn fold(&mut self, id: Id, title: &str, open: &mut bool, body: impl FnOnce(&mut Ui));
    pub fn slider(&mut self, id: Id, v: &mut f64, lo: f64, hi: f64, step: f64) -> bool;
    pub fn number(&mut self, id: Id, v: &mut f64, lo: f64, hi: f64, step: f64) -> bool;
    pub fn text_field(&mut self, id: Id, s: &mut String) -> bool;
    pub fn tabs(&mut self, id: Id, names: &[&str], sel: &mut usize) -> bool;
    pub fn list(&mut self, id: Id, n: usize, row: impl FnMut(&mut Ui, usize)) -> Option<usize>;
    pub fn viewport(&mut self, id: u32, s: Style) -> Resp;   // Cmd::Viewport slot, returns rect + pointer events
    pub fn canvas_image(&mut self, id: ImageId, s: Style) -> Resp;
    pub fn want_redraw(&self) -> bool;                       // animation/caret/progress pending
}
pub struct Theme { pub ground: Color, pub line: Color, pub text: Color, pub muted: Color, pub warn: Color,
                   pub accent: Color, pub font_px: f32, pub gap: f32, pub hairline: f32 }
impl Theme { pub fn dark() -> Self; pub fn light() -> Self; }
```
Frame order: build the tree through closures (layout inputs), call `Tree::compute`, then second pass to hit-test and emit commands. Two passes avoid the "one frame late" problem of naive immediate mode (INTERPRETATION). Widgets keep persistent state only in `HashMap<Id, State>` (scroll, text-field caret/selection, fold open).

**As built (step 9).** The real signatures differ from the sketch above: there is no `Text` trait, because Plex Mono is
monospaced and a text width is arithmetic (`text_width`, `ADVANCE_EM` 0.6); a leaf's measure closure owns its string.
`Ui::new(theme)`, `event(&Event) -> bool`, `begin_frame(size)`, `end_frame() -> &DrawList`; `node(key: &str, style, children)
-> Id` takes a key, not an `Id`, and ids are the hash of the keys down from the root (a repeated key in one frame logs a
warning). `rect(id)` is the widget's absolute rect of the last frame. `button(key, text, ButtonLook) -> Resp` and
`button_with(key, text, look, ButtonOpts { disabled, grow })`; `Resp { id, rect, hovered, held, clicked }`.
`toggle(key, &mut bool, on_word, off_word) -> Resp` takes both words, which come from the application's catalog, and flips
the value before painting. `Theme` holds the page's variables plus `faded(color, opacity)` for CSS opacity.

**As built (step 10).** No slider: neither application has one (see 2.6). `fold(key, title, &mut open, body) -> Resp` and
`fold_with(.., FoldOpts { grow }, ..)`: the widget uppercases the title as the page's CSS does, draws the 1 px rule, the
head (title in the heading look, `+` or `−` at the right, brighter under the pointer) and, only when open, the body (a
closed fold never calls it). The click flips `open` before anything is built, so its own frame shows the new state; the
state is the app's, not a map in `Ui`. `tab(key, text, TabLook::{Strip, Choice}, TabOpts { pressed, disabled }) -> Resp`,
`detail_tab(key, word, sub, opts)`, `strip_gap()`, and the styles `strip_style()`, `choice_style()`, `details_style()`.
The app decides which tab is pressed (the page's strip lights one tab in each of its two groups) and changes its
selection when `Resp.clicked`; there is no `tabs(&[..], &mut usize)`. `number(key, f64) -> Resp` is the hairline box with
the value at the right, formatted as JavaScript would; typing, arrow steps and the focused frame come with step 11.
`Ui::paint_node` paints positioned children after the in-flow ones, as CSS does, so the pressed tab's hairline (position:
relative, margin-left -1) covers its neighbour's grey one. Letter spacing is drawn one glyph per step (`TextLook.tracking`,
`TextLook::width`): Plex Mono is monospaced without kerning, so it equals a spaced string and the renderer is unchanged.
`mix(top, opacity, under)` generalises `Theme::faded`. Hovering the pressed strip tab keeps its word in the ground colour,
unlike the page's cascade (see handoff).
**As built (step 11).** `text_field(key, &mut String, FieldOpts { look: Text | Compact, placeholder, max_chars, width, focus })
-> FieldResp` and `number(key, &mut f64, NumberSpec { min, max, step }) -> FieldResp` (the step-10 `number(key, f64)` is gone;
`NumberSpec::ANY` is the no-limit spec). `FieldResp { resp, focused, changed, submitted, cancelled, blurred }`: the app
keeps its own `String` or `f64` and reads what happened; the widget keeps no per-field map. There is ONE focused field in
the whole `Ui` (`Ui::focus`, with its editing state, made when it gains focus and dropped when it loses it), so a keyed
state map was not needed. `Ui::event` stores `Key` and `Char` events in order and the focused field reads them when it is
built; focus by pointer is decided from the press position against the last frame's rect, before the keys, so a press on
one field and typing right after it work whichever field was built first (a field that lost focus to a later one is told
through a stash, `Ui::lost`). Tab order is build order: Tab commits and blurs the field, and `end_frame` gives the next
field built (wrapping, Shift+Tab backwards) the focus with all its text selected; that happens after painting, so
`Ui::want_redraw()` is true after such a frame and the application draws once more (the widgets demo returns it from
`App::frame`). A focused field that was not built in a frame drops the focus. The editing model is `edit.rs` (`Edit`, byte
indexes on char boundaries, `Outcome`); `Ui::set_clipboard(Box<dyn Clipboard>)` takes the system one (`MemoryClipboard`
by default). A field is one leaf node with `Paint::Field`: frame (text colour when focused, edge otherwise), then clipped
to the content box widened by 1 px, the selection fill, the text, the text again in `select_text` clipped to the selection,
and the 1 px caret (hidden over a selection). Metrics: Text pad 3+1 / 6+1 (26 tall), Compact 1+1 / 6+1 (22), Number 2+1 /
6+1 (24, right-aligned); the glyph box (ascent + descent, 15.6) sits 1.2 under the content top. Pointer: x to column by
`round((x - text_x) / 7.2)`; click counts come from the backend (`PointerDown.clicks`, `GetDoubleClickTime` and
`SM_CXDOUBLECLK`/`SM_CYDOUBLECLK` in `nui-win32`).

### 1.4 nui-win32
Implements `Window`, `Text`, `Gfx`, `Backend`. Internals: `win.rs` (class, wndproc, loop, DPI, clipboard, IME later), `d3d.rs` (device, swap chain, pipelines, meshes, depth), `d2d.rs` (device context, brushes, bitmaps, clip), `dwrite.rs` (factory, text format, layout cache, font loading), `offscreen.rs` (texture + staging readback).

### 1.5 nui-png
```rust
pub fn encode_rgba(w: u32, h: u32, rgba: &[u8]) -> Vec<u8>;
pub fn diff(a: &[u8], b: &[u8]) -> f32;   // mean abs difference, for golden tests
```

---------------------------------------------------------------------------
## 2. Per-module: need, how references do it, our approach, lines

### 2.1 Window and event loop (nui-win32/win.rs, ~450 lines)
- Need: DPI-aware window, on-demand redraw, wake from worker thread.
- References: windows-rs `direct2d` sample (WM_SIZE handling, `ResizeBuffers`, occlusion, device-loss recovery) https://github.com/microsoft/windows-rs/blob/master/crates/samples/windows/direct2d/src/main.rs (ESTABLISHED, read through summary). I found no windows-rs sample with `SetProcessDpiAwarenessContext`/`WM_DPICHANGED` (search returned none); Microsoft docs describe per-monitor v2 (https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process). The `pixelcoords` crate calls `SetProcessDpiAwarenessContext` (https://docs.rs/crate/pixelcoords/0.7.0/source/src/win.rs); a raw FFI binding there, not windows-rs.
- Approach: call `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` before any window (or an application manifest: more robust, UNVERIFIED which is better for a cargo-built exe). `RegisterClassExW`, `CreateWindowExW`, wndproc stores a `*mut State` in `GWLP_USERDATA`. Loop: `GetMessageW` blocks (that gives on-demand for free); wake from other threads with `PostMessageW(hwnd, WM_APP, ..)`. `WM_DPICHANGED` -> `SetWindowPos` to the suggested rect, rebuild D2D target with new scale. Rendering on `WM_PAINT` via `InvalidateRect` (request_redraw). `SetTimer` for the blink. Keep the resize live: render inside `WM_SIZE` (the sample's note says unhandled messages must fall to DefWindowProc so the modal resize loop still sends WM_SIZE).
- Native window frame vs custom chrome: use the standard frame first. The launcher's custom title bar is a later step (`WM_NCCALCSIZE`/`WM_NCHITTEST`; UNVERIFIED effort ~150 lines).

### 2.2 D3D11 + swap chain (d3d.rs, ~600 lines)
- Need: BGRA device, flip-model swap chain, depth buffer, 2 pipelines (textured/colored mesh with MVP + fog/lighting; water alpha blend), resize, offscreen variant.
- References (ESTABLISHED, same sample): device flags always `D3D11_CREATE_DEVICE_BGRA_SUPPORT` (+ DEBUG in debug builds), hardware first and WARP only if `DXGI_ERROR_UNSUPPORTED`; `CreateSwapChainForHwnd` with `DXGI_FORMAT_B8G8R8A8_UNORM`, `BufferCount: 2`, `DXGI_USAGE_RENDER_TARGET_OUTPUT`, `DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL`, no MSAA; resize = drop target bitmap, `ResizeBuffers(0,0,0,DXGI_FORMAT_UNKNOWN,0)`, recreate.
- Approach: same, with `FLIP_DISCARD` optional (UNVERIFIED benefit). Flip model forbids MSAA on the swap chain, so for antialiasing render the 3D into our own MSAA texture and `ResolveSubresource` (skip in v1; the old page used no explicit AA that I know of: UNVERIFIED). Depth: use reverse-Z with `D32_FLOAT`, which replaces three.js's logarithmic depth (INTERPRETATION). Shaders: HLSL strings compiled at startup with `D3DCompile` (`Win32_Graphics_Direct3D_Fxc`, d3dcompiler_47.dll ships with Windows: UNVERIFIED on the target machine) or precompiled `.cso` embedded via `include_bytes!`. Heightfield: CPU-built 512x512 vertex buffer (262k x 24 B ~ 6 MB) and index buffer, colours from the same paint() function used by the 2D map.
- Docs on the interop rules (ESTABLISHED): https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-and-direct3d-interoperation-overview - D2D with D3D11 needs DirectX 11.1+; device must have BGRA support; DXGI-surface render targets cannot Resize (release and recreate, keep at least one D2D resource alive to avoid recreating the device); no DXGI synchronisation is done; formats must match.

### 2.3 D2D on the back buffer (d2d.rs, ~350 lines)
- Need: draw rects, strokes, lines, clip, text, nearest-neighbour bitmap on the swap chain.
- Reference (ESTABLISHED, sample): `D2D1CreateFactory` (single-threaded) -> `factory.CreateDevice(IDXGIDevice)` -> `CreateDeviceContext`; per resize: `swapchain.GetBuffer::<IDXGISurface>(0)` -> `CreateBitmapFromDxgiSurface` with `D2D1_BITMAP_PROPERTIES1 { format B8G8R8A8_UNORM, alphaMode IGNORE, dpi 96, bitmapOptions TARGET | CANNOT_DRAW }` -> `SetTarget(&bitmap)`.
- Approach: use ID2D1DeviceContext (not the old RenderTarget). Set DPI to 96 and do our own scaling: all coordinates in logical px, multiplied by `scale` when emitting, so hairlines can be exactly 1 physical pixel (INTERPRETATION). Per frame: `BeginDraw` ... `EndDraw` per segment between viewports. Brush cache: one `ID2D1SolidColorBrush` re-coloured via `SetColor` (cheap). The 2D map: `ID2D1Bitmap` from RGBA memory, drawn with `DrawBitmap(..., D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR)` (no D3D quad needed; INTERPRETATION, UNVERIFIED that it is pixel-exact for zoomed maps, test it).
- Features of the `windows` crate (see 2.8).

### 2.4 DirectWrite text (dwrite.rs, ~300 lines)
- Need: label drawing/measurement, caret and hit testing, monospace font.
- Approach: `DWriteCreateFactory`, one `IDWriteTextFormat` per size, `CreateTextLayout` cached by (string, size, wrap width) in a small LRU. `DrawTextLayout` with `D2D1_DRAW_TEXT_OPTIONS_NONE`. `caret_x` = `HitTestTextPosition`, `hit` = `HitTestPoint`. Since the font is monospace, `cell()` is measured once ("0" advance), and `caret_x` can be arithmetic for ASCII. Baseline from `IDWriteTextLayout::GetLineMetrics().baseline`. Font loading: IBM Plex Mono may not be installed; options are `IDWriteFontSetBuilder`/in-memory loader (DirectWrite 5, Windows 10 1803+: UNVERIFIED) or `AddFontResourceExW(FR_PRIVATE)` (UNVERIFIED that DirectWrite then sees it). Fall back to Consolas. Decide with a 20-line experiment (build step 3).
- ClearType vs grayscale: set text antialias mode grayscale when drawing over transparent/3D (ClearType needs opaque background; INTERPRETATION).

### 2.5 Layout (nui-layout, ~900 lines) - see section 3 for the algorithm
- References: Taffy `src/compute/flexbox.rs` https://github.com/DioxusLabs/taffy/blob/main/src/compute/flexbox.rs (MIT; ~175k characters, I estimate 3-4k lines, UNVERIFIED), Taffy grid https://github.com/DioxusLabs/taffy/tree/main/src/compute/grid (alignment.rs, explicit_grid.rs, implicit_grid.rs, placement.rs, track_sizing.rs, types/, util/), microui `mu_layout_row` https://github.com/rxi/microui/blob/master/src/microui.c (for contrast: a row model).
- Established from the sources: Taffy's flexbox follows the W3C steps numbered 1..16 (`generate_anonymous_flex_items`, `determine_available_space`, `determine_flex_base_size` including the automatic minimum size, `collect_flex_lines`, `resolve_flexible_lengths`, `determine_hypothetical_cross_size`, `calculate_cross_size`, `determine_used_cross_size`, `distribute_remaining_free_space`, `resolve_cross_axis_auto_margins`, `align_flex_lines_per_align_content`, `final_layout_pass`); baseline via `calculate_children_base_lines`. Taffy's grid track sizing is 11.4-11.8 of the CSS Grid spec: initialize tracks, resolve intrinsic sizes (~230 lines alone), maximise tracks, expand flexible tracks (`find_size_of_fr`), stretch auto tracks. For `minmax(0,1fr)`: min 0 fixed, flexible max, growth limit infinite until step cleanup, final size = leftover / sum of fr factors (at least 1).
- We keep roughly 25% of this (section 3).

### 2.6 Widgets and ui core (nui-ui, ~2200 lines)
- References: microui `mu_update_control`, hover/focus rules (ESTABLISHED, microui.c): hover set when mouse is over and no button down; focus set on press; focus dropped on press elsewhere. egui for IDs and response objects (not read in detail, UNVERIFIED).
- Approach: `Id = hash(parent id, label/index)` (FNV-1a, 10 lines). `Input` struct per frame (pointer, buttons, wheel, keys, chars). Widgets are plain functions building a styled node + emitting commands. Rough sizes: core+ids 300, button/toggle/label 200, fold 120, slider 150, number field 200 (reuses text field), tabs 120, list + scroll area 250, text field 450, viewport/canvas 80, theme 150.
- ~~Slider: map pointer x to value, round by `step` (microui does exactly this).~~ Struck in step 10: neither application has a slider (preview.html has no `input type=range`, its settings are `input type=number`; the launcher has number inputs only; `--track` is defined but used by no rule, and the word "slider" survives only in stale comments). Number field: a text field with a filter (digits, `.`, `-`) that parses on Enter/blur, like microui's `number_textbox` (shift-click to type); we make the field always editable because the page's number inputs are.

### 2.7 Text field (see section 5 below for design)

### 2.8 `windows` crate versions and features
- Latest on crates.io when I checked: 0.62.2, 2025-10-06, license MIT OR Apache-2.0 (https://crates.io/api/v1/crates/windows; ESTABLISHED, but the page was truncated, a newer version may exist: UNVERIFIED).
- The windows-rs `master` sample uses a *different, shorter* feature naming (`"d2d"`, `"d3d11"`, `"dxgi"`, `"winuser"`...) https://github.com/microsoft/windows-rs/blob/master/crates/samples/windows/direct2d/Cargo.toml - so a newer major version has renamed features. The tag 0.58.0 sample uses the classic names (https://github.com/microsoft/windows-rs/blob/0.58.0/crates/samples/windows/direct2d/Cargo.toml). Decision: pin one version exactly (`=0.62.2`) and use the classic names; re-check against the pinned version's docs (https://microsoft.github.io/windows-docs-rs/) before step 2.
- Classic features needed (from the 0.58 sample = ESTABLISHED: `Win32_System_Com`, `Win32_Graphics_Direct2D_Common`, `Win32_Graphics_Direct3D`, `Win32_Graphics_Direct3D11`, `Win32_Graphics_Dxgi_Common`, `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader`, `Win32_UI_WindowsAndMessaging`; I add, UNVERIFIED): `Win32_Foundation`, `Win32_Graphics_Direct2D` (the sample list shows only `_Common`; the parent feature is almost certainly required), `Win32_Graphics_Dxgi`, `Win32_Graphics_DirectWrite`, `Win32_Graphics_Direct3D_Fxc`, `Win32_UI_HiDpi`, `Win32_UI_Input_KeyboardAndMouse`, `Win32_UI_Input_Ime` + `Win32_UI_Input_Ime`/`Win32_Globalization` for IMM32 (phase 2), `Win32_System_DataExchange` + `Win32_System_Memory` + `Win32_System_Ole` (clipboard: `CF_UNICODETEXT` constant location UNVERIFIED), `Win32_Graphics_Imaging` (only if WIC is used, not recommended).

### 2.9 PNG output: recommend our own encoder
- Option A (recommended): ~80-120 lines. Signature `89 50 4E 47 0D 0A 1A 0A`; chunks IHDR (w, h, bit depth 8, colour type 6 RGBA, no interlace), IDAT, IEND; each chunk = length, type, data, CRC32 over type+data. IDAT data is a zlib stream: header `78 01`, then deflate *stored* blocks (BFINAL/BTYPE=00, LEN, NLEN, up to 65535 bytes each), then Adler32 of the raw scanlines. Each scanline is prefixed with filter byte 0. CRC32: 256-entry table, polynomial 0xEDB88320. Adler32: two sums mod 65521. Output size = uncompressed (a 1600x1000 RGBA ~6.4 MB): acceptable for review screenshots; if size matters later add fixed-Huffman deflate with a tiny LZ77 (~120 more lines). Pros: no COM, no Windows dependency, reused unchanged by the Linux backend, deterministic bytes (good for golden tests).
- Option B: WIC (`IWICImagingFactory::CreateEncoder(GUID_ContainerFormatPng)`, `IWICBitmapFrameEncode::WritePixels`), described in Microsoft's article https://learn.microsoft.com/en-us/windows/win32/direct2d/save-direct2d-content-to-an-image-file (ESTABLISHED, C++). Gives compression but adds COM plumbing, a feature flag, and Windows-only code. I found no complete Rust example (search came back empty: UNVERIFIED that none exists).
- Offscreen readback: create `D3D11_TEXTURE2D_DESC { Format: B8G8R8A8_UNORM, BindFlags: RENDER_TARGET | SHADER_RESOURCE, Usage: DEFAULT }`; get its `IDXGISurface` for D2D (same bitmap properties as the back buffer); after drawing, `CopyResource` into a second texture with `Usage: STAGING, CPUAccessFlags: READ`, `Map(.., D3D11_MAP_READ)`, copy row by row honouring `RowPitch`, swap B and R, `Unmap`. Driver: hardware, else WARP (the sample's fallback pattern) so screenshots work without a GPU session. (Pattern: INTERPRETATION from the D3D11 docs; I did not find a Rust example doing exactly this.)

---------------------------------------------------------------------------
## 3. CSS inventory and the reduced layout algorithm

Method: regex over both files, counting each `property: value` (so counts are occurrences in CSS and some JS template strings, not rendered elements). Lines mentioning e.g. `top:` inside JS strings are noise and ignored.

### 3.1 world-preview/page/preview.html
| Property | Values used (count) |
|---|---|
| display | flex (19), grid (13), block (6), none (4 + 1 `!important`) |
| flex-direction | column (10); row = default |
| flex-wrap | wrap (3) |
| flex | `1` (9), `none` (10), `0 1 auto` (1), `1 1 100%` (1) |
| gap | 0, 2px, 4px, 6px, 8px, 10px, 14px, 18px, 20px, 24px, 28px, and row/col pairs `0 16px`, `2px 16px`, `4px 10px`, `4px 14px`; `column-gap: 10px` (3) |
| align-items | baseline (8), center (3), stretch (1) |
| align-self | center (1) |
| justify-content | center (1) |
| place-items | center (2), `start center` (1) |
| order | used inside the media queries to reorder columns/sections (~8 uses) |
| grid-template-columns | `272px minmax(0,1fr) 262px`, `296px minmax(0,1fr) 280px`, `320px minmax(0,1fr) 330px` (the three columns, per breakpoint); `repeat(3, minmax(0,1fr))`; `9em minmax(0,1fr)`; `minmax(0,1fr) auto`; `minmax(0,1fr) auto 8ch`; `minmax(0,1fr) auto auto`; `12px minmax(0,1fr) auto auto`; `12px minmax(0,1fr) 6ch 5ch 5ch 14px` |
| grid-column | `1 / -1` (3) |
| min-width | 0 (5), 5px, 5ch, 6ch, 7ch, 8ch |
| min-height | 0 (8), 32px |
| max-width | 1480px |
| width/height | px, `100%` (13 / 5), `100cqw`, `min(100cqw, 100cqh - var(--under))`, `calc(100cqh - var(--under))`, `calc(100% - 28px)`, `0`, `1px` |
| aspect-ratio | 1 (1), auto (1) |
| position | absolute (10), relative (5); `inset: 0` (5); top/left/right/bottom px, `0`, `calc(100% + 10px)`, `calc((var(--x)+0.5*var(--k))*100%)` |
| overflow | hidden (10), visible (1), `overflow-y: auto` (1) |
| text-overflow / white-space | ellipsis (5) / nowrap (8) |
| margin / padding | `margin: 0 auto`, `4px 0`, etc.; paddings from `0` to `40px`, 1-4 values |
| container-type | `size` (2): on `#stage` and `#flat`; units `cqw`/`cqh` |
| @media | `(max-width: 1250px)` x2, `(max-width: 1100px)` x2, `(prefers-color-scheme: light)` |

### 3.2 outpost-tools/index.html
| Property | Values used |
|---|---|
| display | flex (16), grid (8), block (3), none (4 + 1 important) |
| flex-direction | column (4) |
| flex | `1` (6), `none` (4) |
| gap | 0, 8px, 10px, 12px, 16px, 28px, 48px |
| align-items | center (7), stretch (3), start (1); `align-self: stretch`; `justify-self: end`; `justify-content: center` |
| place-items | center (5) |
| order | several |
| grid-template-columns | `minmax(0,1fr)`, `minmax(0,1fr) minmax(0,1fr)`, `minmax(0,1fr) auto minmax(0,1fr)`, `1fr`, and `0fr` (a collapse transition between `0fr` and `1fr`) |
| sizes | min-width 0 (7) and 8ch; min-height 0, 34px; max-width 1120px; fixed px/%; height 100% |
| position | absolute (2), relative (4), fixed (1); `inset: 0`; top/left/right/bottom px; `z-index: 10` |
| overflow | hidden (5), `overflow-y: auto` (1); text-overflow ellipsis (3), nowrap (4) |
| @media | `(max-width: 1120/860/640px)` style queries (860, 640 confirmed), `prefers-color-scheme`, `prefers-reduced-motion` |

Not used anywhere (so not built): `grid-template-rows`, `grid-template-areas`, `grid-row`, `span`, `repeat(auto-fit/auto-fill)`, `fit-content`, `subgrid`, `flex-basis` other than inside `flex:`, `row`/`column` `-reverse`, `align-content`, `justify-items`, float, `calc` with anything other than `%`/px/cq units, `min-content`/`max-content` keywords, sticky.

### 3.3 Reduced algorithm

Inputs are `Style` structs with the types of section 1.2. Lengths resolve to px before layout (`ch` = monospace cell width x n, `em` = font size x n, cq units and calc are Rust arithmetic).

**Common**
- `layout(node, avail: (Option<f32>, Option<f32>), mode: Measure|Place) -> size`. Memoise per frame on (node, avail) so container-measure + place does not blow up exponentially (Taffy has a cache for the same reason; UNVERIFIED whether our trees need it - they are shallow, keep the cache anyway, ~30 lines).
- Box model: border-box sizing (`box-sizing: border-box` is a standard reset; check the CSS reset line of preview.html: UNVERIFIED). Size = clamp(resolved w/h, min, max); `aspect-ratio` fills the missing axis; `Pct` resolves against the parent's content size, treated as `Auto` when the parent size is indefinite.
- `display: none`: not in the tree. `display: block`: treated as a column flex container with `align-items: stretch`, no gap. Inline text flow beyond a single wrapped text run is NOT supported (risk R3).
- Leaf nodes (text, viewport, image): size comes from the `measure` callback; text uses `Text::measure` with wrap width; `white-space: nowrap` = no wrap, `text-overflow: ellipsis` = the leaf is allowed to shrink (min 0) and draws a truncated string.

**Flex container** (inventory: row/column, nowrap mostly, wrap x3, grow/shrink, basis auto/100%, align stretch/center/start/baseline, justify center, order, margins incl. auto)
1. Collect children: skip `display:none` and `position:absolute`; stable-sort by `order`.
2. Resolve container: inner main/cross available sizes (definite when style or parent gives them).
3. Flex base size per item: `basis` if not Auto, else main-axis `w/h` if definite, else content size by `layout(item, max-content)`. Hypothetical main size = clamp(base, min, max).
4. Automatic minimum: if `min` is Auto and the item's overflow is Visible, min = min(content min-content size, specified size). The page sets `min-width/height: 0` on most flex children precisely to turn this off, so implement: `Auto` => min-content (longest unbreakable run for text, per-child recursion for containers), `Px(0)` => 0.
5. Lines: no-wrap => one line. Wrap => break when sum(hypothetical + margins + gaps) exceeds the container main size (3 uses; row-gap = style.gap.y).
6. Resolve flexible lengths per line (W3C 9.7): free = main - sum(outer base) - gaps; use `grow` if free > 0 else `shrink` weighted by base; loop: compute targets, clamp to min/max, if total violation > 0 freeze the min-violators, < 0 freeze the max-violators, repeat (<= items iterations). ~40 lines. Taffy: `resolve_flexible_lengths`.
7. Cross size per item: hypothetical cross = `layout(item, main = resolved, cross = avail)`. Line cross size = container cross if single-line and definite, else max of item outer cross.
8. Align cross: `align_self.unwrap_or(align_items)`: Start / Center / End as offsets; Stretch sets cross size = line cross - margins when cross is Auto; Baseline: line baseline = max(ascent), each baseline item offset = line baseline - its ascent (used 8x in preview.html for label + number rows; needs `Measured.baseline`, the first text line's ascent, passed up through containers by taking the first child's).
9. Main axis: free space left after flexing; `auto` margins absorb it first (`margin: 0 auto`), else `justify-content` Start (default) / Center. Place items with `gap.x` between.
10. Container intrinsic size when Auto: main = sum(outer hypothetical) + gaps (or max for column cross), cross = max.
Dropped vs Taffy: `align-content`, `-reverse`, visibility collapse, `flex-basis: content`, `min-content` keyword for container width, flexbox_balance, rounding mode. Edge cases kept: percentages, margins auto, min/max clamps.
Estimated 350 lines.

**Grid container** (inventory: explicit columns only, implicit auto rows, auto-placement, `1 / -1`, place-items, gap)
1. Parse the track list (already a `Vec<Track>` in Rust; `repeat(3, ..)` is a `vec![..; 3]`).
2. Place children in row-major order: cursor (col,row); an item with `col_span_all` takes a whole row and moves the cursor to the next row; others take the next free cell. No spans, no explicit rows, no dense packing.
3. Column sizing (Taffy `track_sizing.rs` steps 11.4-11.8 reduced):
   a. Init: `Px(n)`: base = growth = n. `Auto`: base = 0, growth = infinity. `MinMaxFr0(f)`: base = 0, flexible. `Fr(f)` (bare `1fr`, min auto): base = min-content of items, flexible.
   b. Intrinsic: for each `Auto` track base = max over single-span items of their max-content width (we use max-content for both min and growth: our auto tracks are `auto` columns holding numbers/buttons and that is what browsers produce when space is available; if the sum would overflow, shrink auto tracks toward min-content: INTERPRETATION, 15 lines). Items with `col_span_all` are ignored for sizing.
   c. Free space = container inner width - sum(base of non-flexible tracks) - gaps. If container width indefinite: fr tracks take their max-content, free = 0.
   d. Expand flexible: fr size = free / sum(f) (at least 1 as in Taffy's `find_size_of_fr`); if a flexible track's base is larger than f x fr size, freeze it at base, remove from the pool and recompute (loop). Track size = max(base, f x fr).
   e. Stretch auto tracks only when there are no fr tracks and free space is positive (`justify-content: normal`): split free space equally over `Auto` tracks (Taffy `stretch_auto_tracks`).
   f. `0fr` / `1fr` transition (launcher): animate `f` from 0 to 1 over frames; layout just reads `f` (see risk R5).
4. Row sizing: each row (implicit `auto`) height = max over items of `layout(item, width = column span width)` cross size. If container height is definite and align-content is normal, auto rows stretch equally to fill it (this is how `#stage { display:grid; place-items:start center }` fills; INTERPRETATION of spec 11.8, UNVERIFIED against the page, test it).
5. Alignment inside the cell: `place-items`/`align-items`/`justify-self` default Stretch; Center/Start/End by offset; Baseline per row like flex (8 uses of baseline may be on grid rows: UNVERIFIED which container).
6. Container intrinsic size: sum of rows + gaps, sum of tracks + gaps.
Estimated 250 lines.

**Absolute / relative / fixed** (preview: 10 absolute; launcher: 2 absolute + 1 fixed)
- Out of flow: skipped in steps above. After the container is laid out, for each absolute child: containing block = padding box of the nearest `Relative` or `Absolute` ancestor (or the root for fixed). Width: if `w` Auto and both left and right set => stretch, else content size. Position = inset values (`Px`/`Pct`; calc is precomputed in Rust). `inset: 0` = all four 0. `margin: auto` centering for absolute children: not used (skip). ~70 lines.

**Overflow / scroll**
- `Hidden`: push a clip rect for the node's padding box when drawing; the node's min-auto is 0.
- `Scroll/Auto` (y only: 1 use in each file): node keeps `scroll_y` in `Ui` state, content height from `Tree::content_size`, wheel events adjust and clamp, thin scrollbar thumb drawn when content > viewport. ~80 lines in nui-ui.

**Media / container queries**: no engine. The app function does: `let cols = if w < 1100 { A } else if w < 1250 { B } else { C }` (the three grid-template-columns of preview.html map to exactly these). `container-type: size` + `cqw/cqh`: the app reads the stage rect from the previous frame's layout or from a first small layout pass; simplest is to do two `compute` calls per frame when the stage rect changed (INTERPRETATION).

Total layout: about 900 lines including style types, tree, cache and tests.

---------------------------------------------------------------------------
## 4. Ordered build plan (each step compiles and ends in an offscreen PNG unless marked)

All screenshots are produced by `nui-shot <view-name> <w>x<h> [--scale 1.5] [--theme dark] out.png`. Golden PNGs stored as files and compared with `nui-png::diff` in `cargo test`.

| # | Step | Result checked by PNG (what it must show) | Est. h |
|---|---|---|---|
| 0 | Create workspace, `nui-base` types, CI-less `cargo test` | no PNG; compiles | 1 |
| 1 | `nui-png` encoder + generated 256x256 test image | A gradient from black (left) to white (right) with a 16 px red/blue checker in the top-left quadrant; opens in any viewer; CRC errors would make viewers refuse it | 1.5 |
| 2 | `nui-win32` headless: D3D11 device (hardware, WARP fallback), offscreen BGRA texture, clear colour, staging readback -> PNG | A 400x300 image of one flat colour `#101214`; check pixel (0,0) and B/R order with a second run that clears to pure red `#ff0000` | 2 |
| 3 | D2D device context on that texture, DirectWrite text, font loading experiment | Dark ground, a 1 px hairline rectangle, the text `Seed 0123456789 abcXYZ` in IBM Plex Mono (or Consolas fallback, logged), at scale 1.0 and 2.0; the 2.0 image has 1 px hairlines in *physical* pixels | 3 |
| 4 | Window + wndproc + DPI + flip swap chain + resize + on-demand redraw; same frame as step 3 on screen | Live window; `--shot` of the same frame via the window path vs offscreen path differ by < 1 % mean (diff test). Resizing does not leave stale frames | 3 |
| 5 | 3D pipeline: HLSL, depth, one coloured triangle under D2D text | Triangle (red/green/blue vertices) overlapped by a rectangle drawn with `Cmd::Viewport` order: UI below, triangle, UI overlay on top, proving the segment flush | 3 |
| 6 | Heightfield mesh + orbit camera + water plane from synthetic noise, then real `heights.bin`/`biomes.bin` | Fixed camera: terrain shaded by height, a translucent blue water plane covering low areas; same camera as the three.js page gives visually similar silhouette (compare to a browser screenshot) | 6 |
| 7 | `nui-layout` pure crate: flex (steps 1-10), unit tests on rects; debug dump | No GPU: `cargo test` asserts. Then a debug PNG draws node boxes with names for the preview shell at 1400, 1200, 1000 px width: three columns 320/fluid/330, 296/fluid/280, 272/fluid/262; no overlap | 6 |
| 8 | Grid + absolute + overflow in `nui-layout` | Debug PNG of the gallery (`demos/shell-boxes`, `target/tmp/gallery.png`): the page's own rows and boxes, not invented ones: three `.setting` rows (`minmax(0,1fr) auto 8ch`) with the hint, `#figures`, the ore head and rows (`12px 1fr 6ch 5ch 5ch 14px`), biome rows, `#details` (3 equal columns), `#generate` beside the 34 px `#stopRun`, and the `#frame` stack (`#flat` clipped, `#cutLine` out by 9 px, `#note` centred, `#under` below) and a 90 px scroll box | 5 |
| 9 | `nui-ui` core: Ui, ids, input, draw list, theme dark/light; label, button, toggle | Gallery PNG: labels in text/muted/warn tones, buttons in normal/hover/active (hover injected by the harness), toggle on/off; dark and light variants | 5 |
| 10 | Fold, tabs (strip, choice, details), number field (without typing) | Controls sheet: a closed, a hovered and an open fold with its +/− sign, two strips (Flat and 2D pressed; a coarse map with three disabled), choice and details pressed, number fields right-aligned | 5 |
| 11 | Text field: model + keys + caret + selection + clipboard (no IME) | Done: the fields sheet (`docs/shots/fields-dark.png`, `fields-light.png`): a 13-row script on the seed field (placeholder; click; `1234567890`; caret at 4; selection 2 to 6; copy and paste; deleting; a long text scrolled with the caret at the right edge; Ctrl+Backspace; select all; Escape), and cells for a double click, the profile name focused with all selected, a number stepped, clamped on Enter, and Tab between a seed and a setting | 7 |
| 12 | List with scroll + ellipsis; history row layout | A list taller than its box with a scrollbar thumb at 30 %, rows with truncated text ending in an ellipsis | 3 |
| 13 | Preview shell: three columns with the real settings sections, static values | Side by side with a browser screenshot of the current page: same section order, same column widths at the three breakpoints | 6 |
| 14 | 2D map view: paint() port, image upload, pan/zoom, nav frame | The map of a real run: colours per height/biome identical to the page (pixel diff against a canvas dump from the browser < threshold); nearest-neighbour blocks visible at 8x zoom | 5 |
| 15 | 3D view in the stage (Viewport slot), mouse orbit, view switching | Tab strip with Relief selected; terrain inside the stage rect only, UI around it unclipped | 3 |
| 16 | Cross-section view | Slice at a given x/z with ground and water bands, scale labels | 4 |
| 17 | Data and runs: read `out` folder, spawn the PowerShell script, poll progress, run button with progress bar, export PNG button | Progress bar at 40 % with the stop button visible; after export, a PNG file exists and equals the stage content | 5 |
| 18 | Figures column, history list, light/dark switch with OS setting | Right column filled with real numbers; light theme PNG | 3 |
| 19 | Polish: cursor shapes, device loss, DPI change, occlusion, errors | Manual checks at 100/150/200 % | 4 |
| 20 | IME (IMM32) in the text field | Manual only (needs a Japanese/Chinese IME; UNVERIFIED need) | 6 |
| 21 | Launcher migration (custom frame, tabs, collapse animation) in the pack's repository | PNGs of Home and preview tabs | later |

Subtotal steps 0-19 ~ 80 h by my own table here, higher than my earlier ~55 h because this includes the toolkit crate boundaries, golden-image harness and first-class DPI. The earlier figure assumed no separate crates and less testing (INTERPRETATION).

---------------------------------------------------------------------------
## 5. Text field design (egui + microui read)

References: egui `text_selection/text_cursor_state.rs` https://github.com/emilk/egui/blob/master/crates/egui/src/text_selection/text_cursor_state.rs (MIT/Apache, ESTABLISHED by reading): `TextCursorState { ccursor_range: Option<CCursorRange>, granular_drag }`, `CCursorRange { primary (moving), secondary (fixed) }`; positions are *char* offsets, with conversions to byte indexes; `SelectGranularity` Char/Word/Line from click count (1, 2, 3); `is_word_char` = alphanumeric or `_`; word moves via `split_word_bound_indices` and `graphemes(true)` (that is the `unicode-segmentation` crate which we avoid). The key-to-action mapping is NOT in this file (UNVERIFIED location, probably `widgets/text_edit/`). microui `mu_textbox_raw` https://github.com/rxi/microui/blob/master/src/microui.c (ESTABLISHED): focus held after release, appends typed text at the end only, backspace skips UTF-8 continuation bytes, caret = 1 px rect after the text, text shifted left when wider than the box so the end is visible, Enter submits. microui has no caret movement or selection: we need the egui side.

Data:
```rust
struct FieldState { cursor: usize /* byte, on a char boundary */, anchor: usize, scroll_x: f32,
                    blink_phase: bool, drag: Option<Granularity>, undo: Option<(String, usize, usize)> }
```
Selection = [min(cursor,anchor), max). Caret drawn at `Text::caret_x(s, cursor)` minus `scroll_x`.

Keys: Left/Right (shift extends; with a selection and no shift, collapse to that edge), Ctrl+Left/Right (word jump), Home/End, Backspace/Delete (selection first; Ctrl = word), Ctrl+A, Ctrl+C/X/V via `Window::clipboard_*`, Enter (commit, blur), Escape (revert to the value at focus time), Tab (next field), single-level Ctrl+Z (swap with `undo`). Mouse: click places caret via `Text::hit`; drag extends; double click selects word; triple click selects all (single line = egui's Line granularity); shift-click extends. Click count comes in `Event::PointerDown.clicks` from the OS double-click time/distance (`GetDoubleClickTime`, `GetSystemMetrics(SM_CXDOUBLECLK)`).

Grapheme handling without a crate - what is enough: seed fields accept digits, letters and `-`; number fields only `0-9 . - + e`. For those, char == grapheme. Safe subset for arbitrary pasted text (INTERPRETATION): step by Rust `char`, and treat as part of the previous cluster any char in U+0300-U+036F (combining marks), U+200D (zero width joiner) and the following char, U+FE00-U+FE0F (variation selectors), and U+1F3FB-U+1F3FF (skin tones); a surrogate pair is already one `char` in Rust. Wrong for Hangul jamo, Indic scripts, flags: acceptable, document it. Word class: `is_alphanumeric() || '_'`, like egui. Line breaks stripped on paste (single line). Never split inside a char: all moves use `char_indices` / `is_char_boundary`.

IME (phase 2, UNVERIFIED need): IMM32 messages `WM_IME_STARTCOMPOSITION`, `WM_IME_COMPOSITION` (`ImmGetContext`, `ImmGetCompositionStringW` with `GCS_COMPSTR`/`GCS_RESULTSTR`), `ImmSetCompositionWindow` / `ImmSetCandidateWindow` positioned at the caret; draw the preedit string underlined. TSF is the modern alternative but COM-heavy (https://learn.microsoft.com/en-us/windows/win32/tsf/text-services-framework). The only Microsoft sample I found is UWP (`CustomEditControl`, https://github.com/Amka78/Windows-universal-samples/blob/master/Samples/CustomEditControl) and it is stated not to handle surrogates or graphemes properly. For a seed/number field IME has no practical use: defer.

Lines: model + keys 250, mouse/selection 100, drawing/scroll 100.

**Built differently from the plan above (step 11).**
- Steps are by Rust `char`, not by grapheme cluster, and the combining-mark/ZWJ/variation-selector rule was not added: combining marks and ZWJ emoji move one code point at a time (a documented limit).
- No shift-click extension, and no word or line granular drag after a double or triple click (a drag always extends by chars).
- The caret is steady: no blink, so frames are deterministic. Blink goes to step 19.
- Undo is a single coalesced level (typing or deleting runs are one step; paste, cut and edits over a selection are their own), and a second Ctrl+Z redoes. No multi-level history.
- Escape: a text field keeps what was typed (as the browser does) and only blurs; a number field puts back its value at focus. The plan said "revert" for every field.
- An unreadable number buffer ("e", empty) reverts to the value the field had at focus on Enter, Tab or blur. The page falls back to the pack's value there; we have no pack here.
- Line breaks and tabs are removed from a paste (the plan said strip too). UNVERIFIED against Chrome, which may turn a line break into a space.
- `maxlength` counts chars, where a browser counts UTF-16 units.
- The selection colours (`#0078d7`, `#ffffff`, Windows' Highlight pair) are UNVERIFIED against Chrome: the page has no `::selection` rule.
- No IME, no focus ring on buttons (step 19), no textarea (step 21), no hover look on fields (the page has none), no smaller field for the ore rows (step 13).
- `Key` only has the keys the interface acts on, and the letters A, C, V, X, Z only exist as Ctrl shortcuts: the backend sends them with Ctrl held and Alt up.

---------------------------------------------------------------------------
## 6. Risks and open questions

R1. Font: IBM Plex Mono may not be installed; private font loading into DirectWrite is UNVERIFIED (step 3 experiment). Fallback Consolas changes pixel metrics, so goldens must record the font used.
R2. `windows` crate version/feature names changed between 0.58 (classic `Win32_*`) and master (short names). Pin a version; first compile in step 2 will reveal missing features. The list in 2.8 beyond the 0.58 sample is UNVERIFIED.
R3. Inline text flow: `<small>`, `<output>` inside paragraphs, wrapped paragraphs with mixed styles. The reduced layout supports one text run per leaf. I did not read the whole page; check the HTML body (lines 370-440 and JS-built nodes) for mixed inline content before step 13.
R4. Baseline alignment (8 uses in preview). Needs a baseline passed through nested containers; wrong baselines look subtly misaligned. Test with number rows early (step 7).
R5. Launcher `0fr -> 1fr` transition and `prefers-reduced-motion`: needs an animation clock and continuous redraw only while animating (`want_redraw`). Not designed in detail.
R6. D2D `DrawBitmap` nearest-neighbour exactness at fractional scales; fallback is drawing the map as a D3D textured quad (step 14).
R7. Flip-model + D2D: the sample uses `FLIP_SEQUENTIAL` and works (ESTABLISHED); `FLIP_DISCARD` and MSAA through resolve are UNVERIFIED. Device loss/occlusion handling must be copied from the sample (ESTABLISHED there).
R8. Offscreen readback code is my own composition of documented APIs; no Rust reference found (searches returned only C++ docs and crates not showing it). Budget extra time in step 2.
R9. Text rendering differences: DirectWrite ClearType vs the browser's text. Screenshots will never match the page pixel for pixel; compare layout, not glyph pixels.
R10. Time: the earlier ~55 h estimate vs ~80 h in section 4. The gap is mostly the golden-image harness, DPI correctness and crate boundaries. An honest range is 55-85 h; text field and layout are the two items most likely to exceed.
R11. Licenses: Taffy MIT (https://github.com/DioxusLabs/taffy), microui MIT, egui MIT/Apache-2.0 (UNVERIFIED, from memory), Clay Zlib. We copy no code, only algorithms; if any code is copied keep the notice.
R12. Open question for the maintainer: is a custom title bar (launcher) needed in the first toolkit release? It changes the Window trait (hit-test regions) - proposed answer: no, add `Window::set_caption_regions` later.
R13. Open question: should `Gfx` expose raw D3D to the app (preview needs custom shaders) or only `draw_mesh`? Proposed: only `draw_mesh` with two built-in shaders (lit-textured, flat translucent); a Linux backend then needs only two shaders.
R14. Unverified: Taffy line counts, the claim that bare `1fr` should be `minmax(auto, 1fr)` in the preview (page uses `minmax(0,1fr)` everywhere except the launcher's `1fr`/`0fr`).
