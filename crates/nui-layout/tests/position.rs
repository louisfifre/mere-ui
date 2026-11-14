//! Rects asserted for absolute boxes and overflow. Every expectation is worked out by hand from the CSS rule
//! in the test's comment, not copied from the output. Text is IBM Plex Mono as the page sets it: advance
//! 0.6 em, line 1.5 em, baseline 1.125 em (UNVERIFIED against the font).

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

fn fixed(t: &mut Tree, w: f32, h: f32) -> NodeId {
    let n = t.new_node(Style::default());
    t.set_leaf(n, Box::new(move |_| Measured { size: Vec2::new(w, h), baseline: h }));
    n
}

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

/// Greedy word wrap at 12 px.
fn mono(t: &mut Tree, s: &str, wrap: bool) -> NodeId {
    let text = s.to_string();
    let n = t.new_node(Style::default());
    t.set_leaf(
        n,
        Box::new(move |a: Avail| {
            let adv = 7.2;
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
            Measured { size: Vec2::new(longest as f32 * adv, lines as f32 * 18.0), baseline: 13.5 }
        }),
    );
    n
}

/// A positioned parent of a given size with one absolute child; returns (tree, parent, child).
fn with_abs(w: f32, h: f32, child: Style) -> (Tree, NodeId, NodeId) {
    let mut t = Tree::new();
    let parent = t.new_node(Style { position: Position::Relative, ..px(w, h) });
    let kid = t.new_node(Style { position: Position::Absolute, ..child });
    t.append(parent, kid);
    t.compute(parent, Vec2::new(w, h));
    (t, parent, kid)
}

#[test]
fn inset_zero_fills_the_parent() {
    // #flat (line 228): position absolute; inset 0 in the 196 x 196 frame: left, right, top, bottom all 0.
    let (t, _, k) = with_abs(196.0, 196.0, Style { inset: Insets::all(Dim::Px(0.0)), ..Style::default() });
    near(t.rect(k), r(0.0, 0.0, 196.0, 196.0));
}

#[test]
fn left_and_right_stretch_even_when_negative() {
    // #cutLine (line 253): left -9, right -9, top 98, height 0. Width = 196 + 9 + 9 = 214, starting at -9.
    let st = Style {
        inset: Insets { left: Dim::Px(-9.0), right: Dim::Px(-9.0), top: Dim::Px(98.0), bottom: Dim::Auto },
        h: Dim::Px(0.0),
        ..Style::default()
    };
    let (t, _, k) = with_abs(196.0, 196.0, st);
    near(t.rect(k), r(-9.0, 98.0, 214.0, 0.0));
}

#[test]
fn a_box_below_its_parent_takes_its_content_height() {
    // #under (line 260): left 0, right 0, top 196 + 10 = 206, a column with gap 6 holding a 20 and a 23 high
    // child. Width stretches to 196; height has no top/bottom pair, so it is the content: 20 + 6 + 23 = 49.
    let mut t = Tree::new();
    let frame = t.new_node(Style { position: Position::Relative, ..px(196.0, 196.0) });
    let under = t.new_node(Style {
        position: Position::Absolute,
        column: true,
        gap: Vec2::new(0.0, 6.0),
        inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(206.0), bottom: Dim::Auto },
        ..Style::default()
    });
    let a = fixed(&mut t, 10.0, 20.0);
    let b = fixed(&mut t, 10.0, 23.0);
    t.append(frame, under);
    t.append(under, a);
    t.append(under, b);
    t.compute(frame, Vec2::new(196.0, 196.0));
    near(t.rect(under), r(0.0, 206.0, 196.0, 49.0));
    // content_size counts it: 206 + 49 = 255 (the frame's own box is 196).
    let c = t.content_size(frame);
    assert!((c.x - 196.0).abs() < 0.01 && (c.y - 255.0).abs() < 0.01, "content {c:?}");
}

#[test]
fn right_and_bottom() {
    // 20 x 10 box, right 4, bottom 6 in 100 x 50: x = 100 - 4 - 20 = 76, y = 50 - 6 - 10 = 34.
    let st = Style { inset: Insets { right: Dim::Px(4.0), bottom: Dim::Px(6.0), ..Insets::default() }, ..px(20.0, 10.0) };
    let (t, _, k) = with_abs(100.0, 50.0, st);
    near(t.rect(k), r(76.0, 34.0, 20.0, 10.0));
}

#[test]
fn shrink_to_fit_when_only_left_is_set() {
    // Text 80 wide at most, 30 at least; left 10 in a 50 wide parent: the room is 50 - 10 = 40, so it wraps to 40.
    let mut t = Tree::new();
    let parent = t.new_node(Style { position: Position::Relative, ..px(50.0, 50.0) });
    let kid = wrapping(&mut t, 80.0, 30.0);
    *t.style_mut(kid) = Style { position: Position::Absolute, inset: Insets { left: Dim::Px(10.0), ..Insets::default() }, ..Style::default() };
    t.append(parent, kid);
    t.compute(parent, Vec2::new(50.0, 50.0));
    let k = t.rect(kid);
    assert!((k.x - 10.0).abs() < 0.01 && (k.w - 40.0).abs() < 0.01, "got {k:?}");
}

#[test]
fn percent_insets() {
    // left 50 % of the 200 wide parent = 100; top 25 % of its 100 height = 25.
    let st = Style { inset: Insets { left: Dim::Pct(50.0), top: Dim::Pct(25.0), ..Insets::default() }, ..px(10.0, 10.0) };
    let (t, _, k) = with_abs(200.0, 100.0, st);
    near(t.rect(k), r(100.0, 25.0, 10.0, 10.0));
}

#[test]
fn absolute_children_leave_the_flow() {
    // A column whose first child is absolute and second a 50 x 20 box: the box starts at y 0 and the column,
    // sized by content, is 20 high; the absolute box takes no room.
    let mut t = Tree::new();
    let root = t.new_node(Style { align_items: Align::Start, ..Style::default() });
    let col = t.new_node(Style { column: true, position: Position::Relative, ..Style::default() });
    let abs = t.new_node(Style { position: Position::Absolute, inset: Insets::all(Dim::Px(0.0)), ..Style::default() });
    let boxed = t.new_node(px(50.0, 20.0));
    t.append(root, col);
    t.append(col, abs);
    t.append(col, boxed);
    t.compute(root, Vec2::new(200.0, 100.0));
    near(t.rect(boxed), r(0.0, 0.0, 50.0, 20.0));
    near(t.rect(col), r(0.0, 0.0, 50.0, 20.0));
    near(t.rect(abs), r(0.0, 0.0, 50.0, 20.0));
}

#[test]
fn text_centred_in_an_absolute_grid() {
    // #note (line 257): inset 0, grid, place-items center, padding 40, in the 196 square. Inner room 116.
    // "No preview yet. Choose a seed and generate one." at 7.2 per char: 16 columns fit (116 / 7.2 = 16.1):
    // "No preview yet." (15) / "Choose a seed" (13) / "and generate" (12) / "one." = 4 lines of 18 = 72,
    // the widest line 15 x 7.2 = 108. Centred: x = 40 + (116 - 108) / 2 = 44, y = 40 + (116 - 72) / 2 = 62.
    let mut t = Tree::new();
    let frame = t.new_node(Style { position: Position::Relative, ..px(196.0, 196.0) });
    let note = t.new_node(Style {
        display: Display::Grid,
        position: Position::Absolute,
        inset: Insets::all(Dim::Px(0.0)),
        align_items: Align::Center,
        justify_items: Align::Center,
        pad: Edges::all(40.0),
        ..Style::default()
    });
    let text = mono(&mut t, "No preview yet. Choose a seed and generate one.", true);
    t.append(frame, note);
    t.append(note, text);
    t.compute(frame, Vec2::new(196.0, 196.0));
    near(t.rect(note), r(0.0, 0.0, 196.0, 196.0));
    near(t.rect(text), r(44.0, 62.0, 108.0, 72.0));
}

#[test]
fn an_overflow_hidden_item_may_shrink_to_nothing() {
    // A 100 wide row of two 80 wide boxes: 160 asked, 60 too much. Weighted equally each would lose 30, but the
    // first keeps its automatic minimum (its content, 80), so it stays 80. The second has overflow hidden:
    // its automatic minimum is 0, it takes the remaining 20.
    let mut t = Tree::new();
    let root = t.new_node(Style::default());
    let a = fixed(&mut t, 80.0, 18.0);
    let b = fixed(&mut t, 80.0, 18.0);
    t.style_mut(b).overflow = Overflow::Hidden;
    t.append(root, a);
    t.append(root, b);
    t.compute(root, Vec2::new(100.0, 18.0));
    near(t.rect(a), r(0.0, 0.0, 80.0, 18.0));
    near(t.rect(b), r(80.0, 0.0, 20.0, 18.0));
}

#[test]
fn a_scroll_box_reports_its_content() {
    // `.scroll` (line 134): a block, overflow-y auto, height 90, padding 0 10 28 0, holding 8 rows of 18.
    // Rows stack with no gap: y = 18 * i, 290 wide (300 - 10). The content reaches 8 x 18 + 28 = 172; the
    // box itself stays 90.
    let mut t = Tree::new();
    let scroll = t.new_node(Style {
        display: Display::Block,
        overflow: Overflow::ScrollY,
        pad: Edges::new(0.0, 10.0, 28.0, 0.0),
        h: Dim::Px(90.0),
        ..Style::default()
    });
    let rows: Vec<NodeId> = (0..8).map(|_| mono(&mut t, "Temperature scale", false)).collect();
    for &n in &rows {
        t.append(scroll, n);
    }
    t.compute(scroll, Vec2::new(300.0, 90.0));
    for (i, &n) in rows.iter().enumerate() {
        near(t.rect(n), r(0.0, 18.0 * i as f32, 290.0, 18.0));
    }
    let c = t.content_size(scroll);
    assert!((c.y - 172.0).abs() < 0.01, "content {c:?}");
    assert!((t.rect(scroll).h - 90.0).abs() < 0.01);
}

#[test]
fn content_size_counts_absolute_children() {
    // #frame is 196 high and holds #under at top 206 with a 49 high box: the content reaches 206 + 49 = 255.
    let mut t = Tree::new();
    let frame = t.new_node(Style { position: Position::Relative, ..px(196.0, 196.0) });
    let under = t.new_node(Style {
        position: Position::Absolute,
        inset: Insets { left: Dim::Px(0.0), right: Dim::Px(0.0), top: Dim::Px(206.0), bottom: Dim::Auto },
        ..px(196.0, 49.0)
    });
    t.append(frame, under);
    t.compute(frame, Vec2::new(196.0, 196.0));
    let c = t.content_size(frame);
    assert!((c.y - 255.0).abs() < 0.01 && (c.x - 196.0).abs() < 0.01, "content {c:?}");
}
