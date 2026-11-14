//! Build plan step 7's picture: the world preview's page shell (`world-preview/page/preview.html`), laid out
//! by `nui-layout` at a given window width, and drawn as named boxes. Only the shell is here: the toolbar
//! and the three columns with their big blocks. What is inside the blocks comes with steps 9 to 13.
//!
//! Every number that comes from the page says which CSS line it is read from. A number the page does not
//! fix (a block's height, the toolbar strip's width) is marked PLACEHOLDER: it is only there so the boxes
//! have something to hold, and nothing is checked against it.

use nui_base::{Color, DrawList, Rect, Vec2};
use nui_layout::{Dim, Display, Edges, Margin, NodeId, Overflow, Style, Track, Tracks, Tree};

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
            justify: nui_layout::Justify::Center,
            pad: Edges::new(10.0, 16.0, 0.0, 16.0),
            align_items: nui_layout::Align::Start,
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

/// Draws every node as a frame with its name and size, in window coordinates: a container's label sits
/// at its top right, a leaf's at its top left, so a column and its first child do not write over each
/// other. The estimate of a glyph's width is Plex Mono's 0.6 em advance.
pub fn draw_boxes(tree: &Tree, root: NodeId, width: f32, height: f32) -> DrawList {
    let mut list = DrawList::default();
    list.fill(Rect::new(0.0, 0.0, width, height), Color::hex(0x101214));
    let palette = [0xe0e0e0, 0x6fb3d2, 0xd2a36f, 0x8fc98f];
    let mut placed = Vec::new();
    walk(tree, root, 0, &palette, &mut list, &mut placed);
    list
}

/// `placed` keeps the rectangles of the labels already written: a label that would land on one of them
/// (at 1000 px the right column's label and its first child's were 2 px apart) tries its box's bottom
/// edge, and is left out if that is taken too, so every label that is drawn stays readable.
fn walk(tree: &Tree, n: NodeId, depth: usize, palette: &[u32], list: &mut DrawList, placed: &mut Vec<Rect>) {
    let rect = tree.abs_rect(n);
    let colour = Color::hex(palette[depth % palette.len()]);
    list.fill(rect, colour.with_alpha(0.06));
    list.frame(rect, 1.0, colour);
    let name = tree.name(n);
    let text = format!("{} {}x{}", name, rect.w.round(), rect.h.round());
    let size = 11.0;
    let text_w = text.chars().count() as f32 * size * 0.6;
    if rect.w >= 30.0 && rect.h >= 16.0 {
        let leaf = tree.children(n).is_empty();
        let x = if leaf { rect.x + 4.0 } else { (rect.x + rect.w - text_w - 4.0).max(rect.x + 4.0) };
        // The window's own label goes to the bottom, where the toolbar's does not write over it.
        let top = if depth == 0 { rect.y + rect.h - size - 6.0 } else { rect.y + 2.0 };
        let bottom = rect.y + rect.h - size - 4.0;
        for y in [top, bottom] {
            let label = Rect::new(x - 2.0, y - 1.0, text_w + 4.0, size + 2.0);
            let hit = placed.iter().any(|p| label.x < p.x + p.w && p.x < label.x + label.w && label.y < p.y + p.h && p.y < label.y + label.h);
            if !hit {
                placed.push(label);
                list.text(Vec2::new(x, y), text.clone(), size, colour);
                break;
            }
        }
    }
    // An overflow box cuts what its children draw outside it: the picture shows what the page would.
    let clips = tree.style(n).overflow != Overflow::Visible;
    if clips {
        list.push_clip(rect);
    }
    for &c in tree.children(n) {
        walk(tree, c, depth + 1, palette, list, placed);
    }
    if clips {
        list.pop_clip();
    }
}
