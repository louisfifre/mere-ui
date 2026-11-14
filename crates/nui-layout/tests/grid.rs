//! Rects asserted for the grid subset. Every expectation is worked out by hand from the CSS rule in the
//! test's comment, not copied from the output. Text is IBM Plex Mono as the page sets it: advance 0.6 em,
//! line 1.5 em, baseline 1.125 em (Plex Mono's ascent 1.025 + half the line gap; UNVERIFIED against the font).

use nui_base::{Rect, Vec2};
use nui_layout::*;

fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(x, y, w, h)
}

fn near(a: Rect, b: Rect) {
    let ok = |p: f32, q: f32| (p - q).abs() < 0.01;
    assert!(ok(a.x, b.x) && ok(a.y, b.y) && ok(a.w, b.w) && ok(a.h, b.h), "got {a:?}, wanted {b:?}");
}

fn px(w: f32, h: f32) -> Style {
    Style { w: Dim::Px(w), h: Dim::Px(h), ..Style::default() }
}

/// A leaf of a fixed size.
fn fixed(t: &mut Tree, w: f32, h: f32, baseline: f32) -> NodeId {
    let n = t.new_node(Style::default());
    t.set_leaf(n, Box::new(move |_| Measured { size: Vec2::new(w, h), baseline }));
    n
}

/// A leaf of natural width `max_w` whose longest word is `min_w` wide: it wraps when given less.
fn wrapping(t: &mut Tree, max_w: f32, min_w: f32) -> NodeId {
    let n = t.new_node(Style::default());
    t.set_leaf(
        n,
        Box::new(move |a: Avail| {
            let w = match a.w {
                None => max_w,
                Some(av) => av.min(max_w).max(min_w),
            };
            let lines = (max_w / w).ceil().max(1.0);
            Measured { size: Vec2::new(w, lines * 18.0), baseline: 13.5 }
        }),
    );
    n
}

/// Greedy word wrap, the way a browser breaks a paragraph. `wrap: false` keeps one line.
fn mono(t: &mut Tree, s: &str, size: f32, wrap: bool) -> NodeId {
    let text = s.to_string();
    let n = t.new_node(Style::default());
    t.set_leaf(
        n,
        Box::new(move |a: Avail| {
            if text.is_empty() {
                return Measured::default();
            }
            let adv = 0.6 * size;
            let cols = match a.w {
                Some(av) if wrap => ((av / adv + 1e-3).floor() as usize).max(1),
                _ => usize::MAX,
            };
            let (mut lines, mut cur, mut longest) = (1usize, 0usize, 0usize);
            for word in text.split(' ') {
                let len = word.chars().count();
                if cur > 0 && cur + 1 + len > cols {
                    lines += 1;
                    cur = len;
                } else {
                    cur += if cur > 0 { 1 } else { 0 } + len;
                }
                longest = longest.max(cur);
            }
            Measured { size: Vec2::new(longest as f32 * adv, lines as f32 * 1.5 * size), baseline: 1.125 * size }
        }),
    );
    n
}

fn grid(cols: &[Track], gap: Vec2) -> Style {
    Style { display: Display::Grid, cols: Tracks::new(cols), gap, ..Style::default() }
}

#[test]
fn three_columns_of_the_page() {
    // #page: grid-template-columns: 320px minmax(0,1fr) 330px; gap 28px; padding 18 28 22 28; 1400 wide.
    // Inner width 1400 - 56 = 1344; the middle takes 1344 - 320 - 330 - 2*28 = 638.
    // x: 28, 28 + 320 + 28 = 376, 376 + 638 + 28 = 1042.
    let mut t = Tree::new();
    let st = Style {
        pad: Edges::new(18.0, 28.0, 22.0, 28.0),
        ..grid(&[Track::Px(320.0), Track::MinMaxFr0(1.0), Track::Px(330.0)], Vec2::new(28.0, 28.0))
    };
    let root = t.new_node(st);
    let k: Vec<NodeId> = (0..3).map(|_| t.new_node(Style { h: Dim::Px(100.0), ..Style::default() })).collect();
    for &n in &k {
        t.append(root, n);
    }
    t.compute(root, Vec2::new(1400.0, 900.0));
    near(t.rect(k[0]), r(28.0, 18.0, 320.0, 100.0));
    near(t.rect(k[1]), r(376.0, 18.0, 638.0, 100.0));
    near(t.rect(k[2]), r(1042.0, 18.0, 330.0, 100.0));
}

/// `.setting` (line 164): `grid-template-columns: minmax(0,1fr) auto 8ch`, column-gap 10px, align-items center.
fn setting(t: &mut Tree, label: &str, output: &str) -> (NodeId, [NodeId; 3]) {
    let root = t.new_node(Style {
        align_items: Align::Center,
        ..grid(&[Track::MinMaxFr0(1.0), Track::Auto, Track::Px(57.6)], Vec2::new(10.0, 0.0))
    });
    let l = mono(t, label, 12.0, true);
    let o = mono(t, output, 12.0, false);
    let f = t.new_node(Style { h: Dim::Px(24.0), ..Style::default() });
    for &n in &[l, o, f] {
        t.append(root, n);
    }
    (root, [l, o, f])
}

#[test]
fn auto_takes_max_content_fr0_the_rest() {
    // Width 320, gaps 2 x 10, the field 8ch = 57.6, the output "1.00" = 4 x 7.2 = 28.8 (auto = max-content).
    // The label gets what is left: 320 - 20 - 28.8 - 57.6 = 213.6. Columns start at 0, 213.6 + 10, 223.6 + 28.8 + 10.
    // The row is as tall as the 24 px field; the 18 px (12 x 1.5) label is centred in it: (24 - 18) / 2 = 3.
    let mut t = Tree::new();
    let (root, [l, o, f]) = setting(&mut t, "Elevation boost *", "1.00");
    t.compute(root, Vec2::new(320.0, 24.0));
    near(t.rect(l), r(0.0, 3.0, 213.6, 18.0));
    near(t.rect(o), r(223.6, 3.0, 28.8, 18.0));
    near(t.rect(f), r(262.4, 0.0, 57.6, 24.0));
}

#[test]
fn an_empty_auto_track_is_zero_but_keeps_its_gaps() {
    // No output text: the auto column is 0 wide but both 10 px gaps stay: 320 - 20 - 0 - 57.6 = 242.4,
    // the output sits at 242.4 + 10 = 252.4 with width 0.
    let mut t = Tree::new();
    let (root, [l, o, f]) = setting(&mut t, "Elevation boost *", "");
    t.compute(root, Vec2::new(320.0, 24.0));
    near(t.rect(l), r(0.0, 3.0, 242.4, 18.0));
    near(t.rect(o), r(252.4, 12.0, 0.0, 0.0));
    near(t.rect(f), r(262.4, 0.0, 57.6, 24.0));
}

#[test]
fn three_equal_columns_with_a_negative_margin() {
    // #details (lines 195-196) + `.choice` (line 147): three minmax(0,1fr), gap 4, padding-left 1, 320 wide,
    // buttons with margin-left -1. Inner width 319; each column (319 - 2 x 4) / 3 = 103.667.
    // A stretched item fills its column plus the 1 px its margin gives back: 104.667.
    // x = pad 1 + column start - 1 = column start: 0, 103.667 + 4 = 107.667, 215.333; the last one ends at 320.
    let mut t = Tree::new();
    let st = Style {
        pad: Edges::new(0.0, 0.0, 0.0, 1.0),
        align_items: Align::Baseline,
        ..grid(&[Track::MinMaxFr0(1.0); 3], Vec2::new(4.0, 0.0))
    };
    let root = t.new_node(st);
    let k: Vec<NodeId> = (0..3)
        .map(|_| {
            let n = t.new_node(Style { h: Dim::Px(20.0), margin: Margin { left: Side::Px(-1.0), ..Margin::all(0.0) }, ..Style::default() });
            t.append(root, n);
            n
        })
        .collect();
    t.compute(root, Vec2::new(320.0, 20.0));
    near(t.rect(k[0]), r(0.0, 0.0, 104.667, 20.0));
    near(t.rect(k[1]), r(107.667, 0.0, 104.667, 20.0));
    near(t.rect(k[2]), r(215.333, 0.0, 104.667, 20.0));
    let last = t.rect(k[2]);
    assert!((last.x + last.w - 320.0).abs() < 0.01);
}

#[test]
fn span_all_takes_a_row_of_its_own() {
    // Three columns of 100, gap 10: items [a, span-all, b]. a takes row 0 column 0; the span-all item
    // starts a new row and takes all columns (3 x 100 + 2 x 10 = 320); b starts the row after, column 0.
    // Rows are 10 high, no row gap: y = 0, 10, 20.
    let mut t = Tree::new();
    let root = t.new_node(Style { align_items: Align::Start, ..grid(&[Track::Px(100.0); 3], Vec2::new(10.0, 0.0)) });
    let mk = |t: &mut Tree, all: bool| {
        let n = t.new_node(Style { h: Dim::Px(10.0), col_span_all: all, ..Style::default() });
        t.append(root, n);
        n
    };
    let (a, span, b) = (mk(&mut t, false), mk(&mut t, true), mk(&mut t, false));
    t.compute(root, Vec2::new(320.0, 30.0));
    near(t.rect(a), r(0.0, 0.0, 100.0, 10.0));
    near(t.rect(span), r(0.0, 10.0, 320.0, 10.0));
    near(t.rect(b), r(0.0, 20.0, 100.0, 10.0));
}

#[test]
fn a_span_all_hint_goes_under_the_row() {
    // The `.setting` hint (line 171): `small` with grid-column 1 / -1, 11 px, padding 2 0 4. One line is
    // 11 x 1.5 = 16.5, plus 6 padding = 22.5. It sits under the 24 px row, so at y 24, as wide as all
    // columns and gaps: 242.4 + 0 + 57.6 + 20 = 320.
    let mut t = Tree::new();
    let (root, _) = setting(&mut t, "Rolling hills", "");
    let hint = mono(&mut t, "Some plains get very smooth, hilly ground.", 11.0, true);
    *t.style_mut(hint) = Style { pad: Edges::new(2.0, 0.0, 4.0, 0.0), col_span_all: true, ..Style::default() };
    t.append(root, hint);
    t.compute(root, Vec2::new(320.0, 46.5));
    near(t.rect(hint), r(0.0, 24.0, 320.0, 22.5));
}

#[test]
fn a_row_is_as_tall_as_its_tallest_item_and_items_stretch() {
    // Heights 10 and 30 in one row, align-items stretch: the row is 30, both items fill it.
    let mut t = Tree::new();
    let root = t.new_node(grid(&[Track::Px(50.0), Track::Px(50.0)], Vec2::new(0.0, 0.0)));
    let a = fixed(&mut t, 5.0, 10.0, 8.0);
    let b = fixed(&mut t, 5.0, 30.0, 8.0);
    t.append(root, a);
    t.append(root, b);
    t.compute(root, Vec2::new(100.0, 30.0));
    near(t.rect(a), r(0.0, 0.0, 50.0, 30.0));
    near(t.rect(b), r(50.0, 0.0, 50.0, 30.0));
}

#[test]
fn baseline_rows() {
    // `.ore-row` (line 179): columns 12px, minmax(0,1fr), 6ch, 5ch, 5ch, 14px at 12 px (ch = 7.2): 12, fr,
    // 43.2, 36, 36, 14; gap 10; align-items baseline; 320 wide.
    // Inputs: padding 2 5, 12 px text: height 18 + 4 = 22, baseline 13.5 + 2 = 15.5. Name and button: height
    // 18, baseline 13.5. The swatch (12 x 12) is align-self center.
    // Above the baseline: max(15.5, 13.5) = 15.5; below: max(22 - 15.5, 18 - 13.5) = 6.5; the row is 22.
    // Name y = 15.5 - 13.5 = 2; inputs y = 0; swatch (22 - 12) / 2 = 5.
    // x: 0, 12 + 10 = 22, fr = 320 - (12 + 43.2 + 36 + 36 + 14) - 50 = 128.8, then 22 + 128.8 + 10 = 160.8,
    // 160.8 + 43.2 + 10 = 214, 214 + 36 + 10 = 260, 260 + 36 + 10 = 306.
    let mut t = Tree::new();
    let cols = [Track::Px(12.0), Track::MinMaxFr0(1.0), Track::Px(43.2), Track::Px(36.0), Track::Px(36.0), Track::Px(14.0)];
    let root = t.new_node(Style { align_items: Align::Baseline, ..grid(&cols, Vec2::new(10.0, 0.0)) });
    let swatch = t.new_node(Style { align_self: Some(Align::Center), ..px(12.0, 12.0) });
    let name = mono(&mut t, "Coal", 12.0, true);
    let input = |t: &mut Tree, s: &str| {
        let n = mono(t, s, 12.0, false);
        t.style_mut(n).pad = Edges::new(2.0, 5.0, 2.0, 5.0);
        n
    };
    let (i1, i2, i3) = (input(&mut t, "2"), input(&mut t, "8"), input(&mut t, "32"));
    let button = mono(&mut t, "x", 12.0, false);
    for &n in &[swatch, name, i1, i2, i3, button] {
        t.append(root, n);
    }
    t.compute(root, Vec2::new(320.0, 22.0));
    near(t.rect(swatch), r(0.0, 5.0, 12.0, 12.0));
    near(t.rect(name), r(22.0, 2.0, 128.8, 18.0));
    near(t.rect(i1), r(160.8, 0.0, 43.2, 22.0));
    near(t.rect(i2), r(214.0, 0.0, 36.0, 22.0));
    near(t.rect(i3), r(260.0, 0.0, 36.0, 22.0));
    near(t.rect(button), r(306.0, 2.0, 14.0, 18.0));

    // #figures (line 297): a 14 px name and a 12 px value in one baseline row. Above: max(15.75, 13.5) = 15.75;
    // below: max(21 - 15.75, 18 - 13.5) = 5.25; row 21. The 12 px text sits at 15.75 - 13.5 = 2.25.
    let mut t = Tree::new();
    let root = t.new_node(Style { align_items: Align::Baseline, ..grid(&[Track::Px(100.0), Track::Px(100.0)], Vec2::new(14.0, 4.0)) });
    let big = mono(&mut t, "Flat land", 14.0, true);
    let small = mono(&mut t, "38 %", 12.0, false);
    t.append(root, big);
    t.append(root, small);
    t.compute(root, Vec2::new(214.0, 21.0));
    near(t.rect(big), r(0.0, 0.0, 100.0, 21.0));
    near(t.rect(small), r(114.0, 2.25, 100.0, 18.0));
}

#[test]
fn a_definite_height_stretches_the_row_and_centres() {
    // #stopRun (line 219): 34 x 34 grid, no template, place-items center, a 12 x 12 child.
    // The one row takes the 34 px height, the one column the 34 px width: (34 - 12) / 2 = 11 both ways.
    let mut t = Tree::new();
    let root = t.new_node(Style { display: Display::Grid, align_items: Align::Center, justify_items: Align::Center, ..px(34.0, 34.0) });
    let icon = t.new_node(px(12.0, 12.0));
    t.append(root, icon);
    t.compute(root, Vec2::new(34.0, 34.0));
    near(t.rect(icon), r(11.0, 11.0, 12.0, 12.0));
}

#[test]
fn justify_centre_keeps_a_sized_item_whole() {
    // #stage (line 226): place-items start center, 330 x 260. A child 196 wide with aspect-ratio 1 is 196 high,
    // at the top (align start) and centred across: (330 - 196) / 2 = 67.
    let mut t = Tree::new();
    let root = t.new_node(Style { display: Display::Grid, align_items: Align::Start, justify_items: Align::Center, ..px(330.0, 260.0) });
    let frame = t.new_node(Style { w: Dim::Px(196.0), aspect: Some(1.0), ..Style::default() });
    t.append(root, frame);
    t.compute(root, Vec2::new(330.0, 260.0));
    near(t.rect(frame), r(67.0, 0.0, 196.0, 196.0));
}

#[test]
fn auto_tracks_share_a_short_room_from_min_content() {
    // Two auto columns, 100 wide; each text is 80 wide at most and 30 at least. Bases are the min-contents
    // (30 + 30), the 40 left is shared equally (20 each): 50 and 50, both under their max-content.
    let mut t = Tree::new();
    let root = t.new_node(grid(&[Track::Auto, Track::Auto], Vec2::new(0.0, 0.0)));
    let a = wrapping(&mut t, 80.0, 30.0);
    let b = wrapping(&mut t, 80.0, 30.0);
    t.append(root, a);
    t.append(root, b);
    t.compute(root, Vec2::new(100.0, 40.0));
    near(t.rect(a), r(0.0, 0.0, 50.0, t.rect(a).h));
    near(t.rect(b), r(50.0, 0.0, 50.0, t.rect(b).h));
}

#[test]
fn a_grid_sizes_itself_in_a_row() {
    // A flex row 500 wide holds a grid with no width: columns 12px, minmax(0,1fr), auto, gap 10.
    // Max-content: 12 + 100 (the fr column's item) + 40 + 2 x 10 = 172, under the 500 offered, so 172.
    // The fr column then gets 172 - 12 - 40 - 20 = 100.
    let mut t = Tree::new();
    let root = t.new_node(Style { align_items: Align::Start, ..Style::default() });
    let g = t.new_node(grid(&[Track::Px(12.0), Track::MinMaxFr0(1.0), Track::Auto], Vec2::new(10.0, 0.0)));
    t.append(root, g);
    let a = wrapping(&mut t, 12.0, 12.0);
    let b = wrapping(&mut t, 100.0, 100.0);
    let c = wrapping(&mut t, 40.0, 40.0);
    for &n in &[a, b, c] {
        t.append(g, n);
    }
    t.compute(root, Vec2::new(500.0, 100.0));
    assert!((t.rect(g).w - 172.0).abs() < 0.01, "grid width {}", t.rect(g).w);
    near(t.rect(b), r(22.0, 0.0, 100.0, 18.0));
    near(t.rect(c), r(132.0, 0.0, 40.0, 18.0));
}

#[test]
fn order_display_none_and_absolute_children_take_no_cell() {
    // Two columns of 50, gap 10. Children in source order: A (order 1), B (display none), C (absolute),
    // D (order 0). D sorts first and takes column 0; A takes column 1 (x = 50 + 10); B and C take no cell.
    let mut t = Tree::new();
    let root = t.new_node(grid(&[Track::Px(50.0), Track::Px(50.0)], Vec2::new(10.0, 0.0)));
    let a = t.new_node(Style { order: 1, h: Dim::Px(10.0), ..Style::default() });
    let b = t.new_node(Style { display: Display::None, h: Dim::Px(10.0), ..Style::default() });
    let c = t.new_node(Style { position: Position::Absolute, h: Dim::Px(10.0), ..Style::default() });
    let d = t.new_node(Style { h: Dim::Px(10.0), ..Style::default() });
    for &n in &[a, b, c, d] {
        t.append(root, n);
    }
    t.compute(root, Vec2::new(110.0, 10.0));
    near(t.rect(d), r(0.0, 0.0, 50.0, 10.0));
    near(t.rect(a), r(60.0, 0.0, 50.0, 10.0));
    near(t.rect(b), r(0.0, 0.0, 0.0, 0.0));
}

#[test]
fn percent_width_of_an_item_is_of_its_cell() {
    // A 50 % wide item in a 100 px column is 50 px, not half of the grid.
    let mut t = Tree::new();
    let root = t.new_node(grid(&[Track::Px(100.0), Track::Px(100.0)], Vec2::new(0.0, 0.0)));
    let a = t.new_node(Style { w: Dim::Pct(50.0), h: Dim::Px(10.0), ..Style::default() });
    let b = t.new_node(Style { w: Dim::Pct(50.0), h: Dim::Px(10.0), ..Style::default() });
    t.append(root, a);
    t.append(root, b);
    t.compute(root, Vec2::new(200.0, 10.0));
    near(t.rect(a), r(0.0, 0.0, 50.0, 10.0));
    near(t.rect(b), r(100.0, 0.0, 50.0, 10.0));
}
