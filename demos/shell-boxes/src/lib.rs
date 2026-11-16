//! Build plan step 7's picture: the world preview's page shell (`world-preview/page/preview.html`), laid out
//! by `mere-layout` at a given window width, and drawn as named boxes. Only the shell is here: the toolbar
//! and the three columns with their big blocks. What is inside the blocks comes with steps 9 to 13.
//!
//! Every number that comes from the page says which CSS line it is read from. A number the page does not
//! fix (a block's height, the toolbar strip's width) is marked PLACEHOLDER: it is only there so the boxes
//! have something to hold, and nothing is checked against it.

pub mod gallery;

use mere_base::{Color, DrawList, Rect, Vec2};
use mere_layout::{Dim, Display, Edges, Margin, NodeId, Overflow, Style, Track, Tracks, Tree};

/// The three layouts of `#page`, one per media query.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Columns {
    pub left: f32,
    pub right: f32,
    pub gap: f32,
    pub pad: Edges,
}

impl Columns {
    /// `@media (max-width: 1100px)` and `(max-width: 1250px)` both match the window width inclusively, and
    /// the narrower one comes later in the file so it wins when both match (preview.html lines 83-88).
    pub fn for_width(width: f32) -> Columns {
        if width <= 1100.0 {
            // line 87: grid-template-columns: 272px minmax(0, 1fr) 262px; gap: 20px; padding: 14px 16px 18px
            Columns { left: 272.0, right: 262.0, gap: 20.0, pad: Edges::new(14.0, 16.0, 18.0, 16.0) }
        } else if width <= 1250.0 {
            // line 84: grid-template-columns: 296px minmax(0, 1fr) 280px; gap: 24px; padding: 16px 22px 20px
            Columns { left: 296.0, right: 280.0, gap: 24.0, pad: Edges::new(16.0, 22.0, 20.0, 22.0) }
        } else {
            // lines 77-80: padding: 18px 28px 22px; 320px minmax(0, 1fr) 330px; gap: 28px
            Columns { left: 320.0, right: 330.0, gap: 28.0, pad: Edges::new(18.0, 28.0, 22.0, 28.0) }
        }
    }
}

/// The nodes a test wants to look at by name.
pub struct Shell {
    pub tree: Tree,
    pub root: NodeId,
    pub toolbar: NodeId,
    pub strip: NodeId,
    pub page: NodeId,
    pub left: NodeId,
    pub middle: NodeId,
    pub right: NodeId,
    pub stage: NodeId,
    pub under: NodeId,
}

fn named(t: &mut Tree, name: &str, style: Style) -> NodeId {
    let n = t.new_node(style);
    t.set_name(n, name);
    n
}

fn add(t: &mut Tree, parent: NodeId, name: &str, style: Style) -> NodeId {
    let n = named(t, name, style);
    t.append(parent, n);
    n
}

/// `.column { min-height: 0; display: flex; flex-direction: column; gap: 0 }` (line 106).
fn column(extra: Style) -> Style {
    Style { column: true, min_h: Dim::Px(0.0), ..extra }
}

/// `flex: 1; min-height: 0` on a column's child: it takes what the others leave.
fn fills() -> Style {
    Style { grow: 1.0, shrink: 1.0, basis: Dim::Px(0.0), min_h: Dim::Px(0.0), ..Style::default() }
}

/// A block of a fixed height that does not give way (`flex: none`).
fn fixed_h(h: f32) -> Style {
    Style { h: Dim::Px(h), shrink: 0.0, ..Style::default() }
}

/// The page's `body` (a column filling the window) holding the toolbar and `#page`, laid out for a window
/// of `width` x `height` px.
pub fn build(width: f32, height: f32) -> Shell {
    let cols = Columns::for_width(width);
    let mut t = Tree::new();

    // body { display: flex; flex-direction: column } (lines 63-72), the window's size.
    let root = named(&mut t, "body", Style { column: true, ..Style::default() });

    // #toolbar { flex: none; display: flex; justify-content: center; padding: 10px 16px 0 } (line 93);
    // its strip's buttons are 28px high (line 96). The strip's width is PLACEHOLDER: the page builds its
    // buttons from script.
    let toolbar = add(
        &mut t,
        root,
        "toolbar",
        Style {
            shrink: 0.0,
            justify: mere_layout::Justify::Center,
            pad: Edges::new(10.0, 16.0, 0.0, 16.0),
            align_items: mere_layout::Align::Start,
            ..Style::default()
        },
    );
    let strip = add(&mut t, toolbar, "strip", Style { w: Dim::Px(360.0), h: Dim::Px(28.0), shrink: 0.0, ..Style::default() });

    // #page { flex: 1; min-height: 0; width: 100%; max-width: 1480px; margin: 0 auto; display: grid;
    // grid-template-columns: Lpx minmax(0, 1fr) Rpx; gap } (lines 75-81). The gap is on both axes (line 80)
    // but there is one implicit row: it stretches to the page's height (the grid's definite height is shared
    // among its rows), and the three columns stretch into that row.
    let page = add(
        &mut t,
        root,
        "page",
        Style {
            display: Display::Grid,
            cols: Tracks::new(&[Track::Px(cols.left), Track::MinMaxFr0(1.0), Track::Px(cols.right)]),
            grow: 1.0,
            min_h: Dim::Px(0.0),
            w: Dim::Pct(100.0),
            max_w: Dim::Px(1480.0),
            margin: Margin::horizontal_auto(),
            pad: cols.pad,
            gap: Vec2::new(cols.gap, cols.gap),
            ..Style::default()
        },
    );
    // Left column: world fold, tuning fold (the `grow` one), run block. The heights of the first and last
    // are PLACEHOLDER; the tuning fold takes the rest.
    let left = add(&mut t, page, "left column", column(Style::default()));
    add(&mut t, left, "worldFold", fixed_h(130.0));
    add(&mut t, left, "tuningFold", fills());
    add(&mut t, left, "run", fixed_h(150.0));

    // Middle column, `minmax(0, 1fr)`. #stage { flex: 1; min-height: 0 } (line 226). The page keeps
    // `--under: 64px` (same line) free below the map for its #under strip; in the page that strip is
    // inside the stage, here it is a sibling box of that height (an approximation: the stage is only
    // checked down to its column).
    let middle = add(&mut t, page, "middle column", column(Style::default()));
    let stage = add(&mut t, middle, "stage", fills());
    let under = add(&mut t, middle, "under", fixed_h(64.0));

    // Right column (#numbers). Heights of the figures block are PLACEHOLDER; the list takes the rest.
    let right = add(&mut t, page, "right column", column(Style::default()));
    add(&mut t, right, "figuresBlock", fixed_h(170.0));
    add(&mut t, right, "listBlock", fills());

    t.compute(root, Vec2::new(width, height));
    Shell { tree: t, root, toolbar, strip, page, left, middle, right, stage, under }
}

/// Draws every node as a frame, then the labels (name and size) on top. See `label_rects` for where they go.
pub fn draw_boxes(tree: &Tree, root: NodeId, width: f32, height: f32) -> DrawList {
    let mut list = DrawList::default();
    list.fill(Rect::new(0.0, 0.0, width, height), Color::hex(0x101214));
    let palette = [0xe0e0e0, 0x6fb3d2, 0xd2a36f, 0x8fc98f];
    draw_frames(tree, root, 0, &palette, &mut list);
    for l in label_rects(tree, root, width, height) {
        list.text(Vec2::new(l.x, l.y), l.text, LABEL_SIZE, Color::hex(palette[l.depth % palette.len()]));
    }
    list
}

const LABEL_SIZE: f32 = 11.0;
/// Plex Mono's 0.6 em advance, the estimate of a glyph's width.
const LABEL_ADVANCE: f32 = LABEL_SIZE * 0.6;

/// One label as drawn: the text origin, the text (already cut to fit) and the rectangle it occupies.
pub struct Label {
    pub node: NodeId,
    pub depth: usize,
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub rect: Rect,
}

fn draw_frames(tree: &Tree, n: NodeId, depth: usize, palette: &[u32], list: &mut DrawList) {
    let rect = tree.abs_rect(n);
    let colour = Color::hex(palette[depth % palette.len()]);
    list.fill(rect, colour.with_alpha(0.06));
    list.frame(rect, 1.0, colour);
    // An overflow box cuts what its children draw outside it: the picture shows what the page would.
    let clips = tree.style(n).overflow != Overflow::Visible;
    if clips {
        list.push_clip(rect);
    }
    for &c in tree.children(n) {
        draw_frames(tree, c, depth + 1, palette, list);
    }
    if clips {
        list.pop_clip();
    }
}

/// The labels, children before their parent: a content box (an output, a field, a value) keeps its text and a
/// container's label goes only where nothing is taken, at its top right, else its bottom edge, else nowhere.
/// A text is cut to its own box (8 px of margin) and a label is dropped when it would leave the picture or the
/// box of an overflow ancestor, so no label ever crosses an edge.
pub fn label_rects(tree: &Tree, root: NodeId, width: f32, height: f32) -> Vec<Label> {
    let mut out = Vec::new();
    let mut placed = Vec::new();
    collect(tree, root, 0, Rect::new(0.0, 0.0, width, height), &mut placed, &mut out);
    out
}

fn collect(tree: &Tree, n: NodeId, depth: usize, clip: Rect, placed: &mut Vec<Rect>, out: &mut Vec<Label>) {
    let rect = tree.abs_rect(n);
    let inner = if tree.style(n).overflow != Overflow::Visible { intersect(clip, rect) } else { clip };
    for &c in tree.children(n) {
        collect(tree, c, depth + 1, inner, placed, out);
    }
    if rect.w < 30.0 || rect.h < 16.0 {
        return;
    }
    let full = format!("{} {}x{}", tree.name(n), rect.w.round(), rect.h.round());
    let room = ((rect.w - 8.0) / LABEL_ADVANCE).floor().max(0.0) as usize;
    let text: String = full.chars().take(room).collect();
    if text.chars().count() < 3 {
        return;
    }
    let text_w = text.chars().count() as f32 * LABEL_ADVANCE;
    let leaf = tree.children(n).is_empty();
    let x = if leaf { rect.x + 4.0 } else { rect.x + rect.w - text_w - 4.0 };
    // The window's own label goes to the bottom, where the toolbar's does not write over it.
    let top = if depth == 0 { rect.y + rect.h - LABEL_SIZE - 6.0 } else { rect.y + 2.0 };
    let bottom = rect.y + rect.h - LABEL_SIZE - 4.0;
    for y in [top, bottom] {
        let label = Rect::new(x - 2.0, y - 1.0, text_w + 4.0, LABEL_SIZE + 2.0);
        let hit = placed.iter().any(|p| overlaps(label, *p)) || touches_descendant(tree, n, label);
        if !hit && contains(clip_to(inner, rect), label) {
            placed.push(label);
            out.push(Label { node: n, depth, x, y, text, rect: label });
            return;
        }
    }
}

/// True when the label, grown by 1 px, meets the box of any descendant of `n`: a container's label must not sit
/// on its children's frames or on what they draw (a 1 px gap keeps glyph tops and bottoms off the lines).
fn touches_descendant(tree: &Tree, n: NodeId, label: Rect) -> bool {
    let grown = Rect::new(label.x - 1.0, label.y - 1.0, label.w + 2.0, label.h + 2.0);
    tree.children(n).iter().any(|&c| {
        let r = tree.abs_rect(c);
        (r.w > 0.0 || r.h > 0.0) && (overlaps(grown, Rect::new(r.x, r.y, r.w.max(0.01), r.h.max(0.01))) || touches_descendant(tree, c, label))
    })
}

/// The region a node's label may use: its own box cut by what its ancestors clip.
fn clip_to(inner: Rect, rect: Rect) -> Rect {
    intersect(inner, rect)
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn intersect(a: Rect, b: Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    Rect::new(x, y, ((a.x + a.w).min(b.x + b.w) - x).max(0.0), ((a.y + a.h).min(b.y + b.h) - y).max(0.0))
}

fn contains(outer: Rect, r: Rect) -> bool {
    r.x >= outer.x - 0.01 && r.y >= outer.y - 0.01 && r.x + r.w <= outer.x + outer.w + 0.01 && r.y + r.h <= outer.y + outer.h + 0.01
}
