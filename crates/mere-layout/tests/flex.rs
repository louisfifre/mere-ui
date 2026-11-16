//! Rects asserted for the flex subset. Every expectation is worked out by hand from the CSS rule in the
//! test's comment, not copied from the output.

use mere_layout::*;
use mere_base::{Rect, Vec2};

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

/// A one-line text of natural width `max_w` whose longest word is `min_w` wide: it wraps when given less.
fn text(t: &mut Tree, max_w: f32, min_w: f32, line_h: f32, ascent: f32) -> NodeId {
    let n = t.new_node(Style::default());
    t.set_leaf(
        n,
        Box::new(move |a: Avail| {
            let w = match a.w {
                None => max_w,
                Some(av) => av.min(max_w).max(min_w),
            };
            let lines = (max_w / w).ceil().max(1.0);
            Measured { size: Vec2::new(w, lines * line_h), baseline: ascent }
        }),
    );
    n
}

fn tree_with(root_style: Style, kids: &[Style]) -> (Tree, NodeId, Vec<NodeId>) {
    let mut t = Tree::new();
    let root = t.new_node(root_style);
    let ids = kids
        .iter()
        .map(|s| {
            let k = t.new_node(*s);
            t.append(root, k);
            k
        })
        .collect();
    (t, root, ids)
}

#[test]
fn row_places_fixed_boxes_with_a_gap() {
    // display:flex; gap:10px; children 50x20 and 30x40 (align-items: stretch is overridden by their heights)
    let root = Style { gap: Vec2::new(10.0, 0.0), ..Style::default() };
    let (mut t, root, k) = tree_with(root, &[px(50.0, 20.0), px(30.0, 40.0)]);
    t.compute(root, Vec2::new(200.0, 100.0));
    near(t.rect(k[0]), r(0.0, 0.0, 50.0, 20.0));
    near(t.rect(k[1]), r(60.0, 0.0, 30.0, 40.0));
}

#[test]
fn grow_shares_free_space_by_factor() {
    // 300 wide, a 60px fixed box, then flex:1 and flex:2 with basis 0: free = 240 -> 80 and 160
    let grow = |g: f32| Style { grow: g, basis: Dim::Px(0.0), min_w: Dim::Px(0.0), ..Style::default() };
    let (mut t, root, k) = tree_with(Style::default(), &[px(60.0, 10.0), grow(1.0), grow(2.0)]);
    t.compute(root, Vec2::new(300.0, 50.0));
    near(t.rect(k[1]), r(60.0, 0.0, 80.0, 50.0));
    near(t.rect(k[2]), r(140.0, 0.0, 160.0, 50.0));
}

#[test]
fn shrink_is_weighted_by_base_size() {
    // bases 100 and 300 in 300px: overflow 100, shares 100:300 -> they give up 25 and 75
    let (mut t, root, k) = tree_with(Style::default(), &[px(100.0, 10.0), px(300.0, 10.0)]);
    for &n in &k {
        t.style_mut(n).min_w = Dim::Px(0.0);
    }
    t.compute(root, Vec2::new(300.0, 10.0));
    near(t.rect(k[0]), r(0.0, 0.0, 75.0, 10.0));
    near(t.rect(k[1]), r(75.0, 0.0, 225.0, 10.0));
}

#[test]
fn a_max_width_freezes_the_item_and_the_rest_takes_what_is_left() {
    // two flex:1 boxes in 400px, the first capped at 100: it takes 100, the other 300
    let one = Style { grow: 1.0, basis: Dim::Px(0.0), ..Style::default() };
    let capped = Style { max_w: Dim::Px(100.0), ..one };
    let (mut t, root, k) = tree_with(Style::default(), &[capped, one]);
    t.compute(root, Vec2::new(400.0, 20.0));
    near(t.rect(k[0]), r(0.0, 0.0, 100.0, 20.0));
    near(t.rect(k[1]), r(100.0, 0.0, 300.0, 20.0));
}

#[test]
fn a_min_width_freezes_a_shrinking_item() {
    // 100 + 100 in 120px; the first has min-width 90: it stays 90, the second gets 30
    let (mut t, root, k) = tree_with(Style::default(), &[px(100.0, 10.0), px(100.0, 10.0)]);
    t.style_mut(k[0]).min_w = Dim::Px(90.0);
    t.style_mut(k[1]).min_w = Dim::Px(0.0);
    t.compute(root, Vec2::new(120.0, 10.0));
    near(t.rect(k[0]), r(0.0, 0.0, 90.0, 10.0));
    near(t.rect(k[1]), r(90.0, 0.0, 30.0, 10.0));
}

#[test]
fn flex_none_does_not_shrink() {
    // flex: none = 0 0 auto
    let none = Style { shrink: 0.0, ..px(200.0, 10.0) };
    let (mut t, root, k) = tree_with(Style::default(), &[none]);
    t.compute(root, Vec2::new(100.0, 10.0));
    near(t.rect(k[0]), r(0.0, 0.0, 200.0, 10.0));
}

#[test]
fn column_stretches_across_and_stacks_down() {
    // flex-direction: column; padding 5; gap 4; children without width stretch to the content width
    let root = Style { column: true, gap: Vec2::new(0.0, 4.0), pad: Edges::all(5.0), ..Style::default() };
    let tall = Style { h: Dim::Px(20.0), ..Style::default() };
    let (mut t, root, k) = tree_with(root, &[tall, Style { h: Dim::Px(30.0), w: Dim::Px(40.0), ..Style::default() }]);
    t.compute(root, Vec2::new(100.0, 100.0));
    near(t.rect(k[0]), r(5.0, 5.0, 90.0, 20.0));
    near(t.rect(k[1]), r(5.0, 29.0, 40.0, 30.0));
}

#[test]
fn column_grow_fills_the_height() {
    // the page's `flex: 1; min-height: 0` under a fixed-height bar
    let (mut t, root, k) = tree_with(Style { column: true, ..Style::default() }, &[
        Style { h: Dim::Px(38.0), shrink: 0.0, ..Style::default() },
        Style { grow: 1.0, min_h: Dim::Px(0.0), ..Style::default() },
    ]);
    t.compute(root, Vec2::new(500.0, 400.0));
    near(t.rect(k[0]), r(0.0, 0.0, 500.0, 38.0));
    near(t.rect(k[1]), r(0.0, 38.0, 500.0, 362.0));
}

#[test]
fn align_items_center_end_and_align_self() {
    let root = Style { align_items: Align::Center, ..Style::default() };
    let (mut t, root, k) = tree_with(root, &[px(20.0, 10.0), px(20.0, 30.0), px(20.0, 10.0)]);
    t.style_mut(k[2]).align_self = Some(Align::End);
    t.compute(root, Vec2::new(100.0, 50.0));
    near(t.rect(k[0]), r(0.0, 20.0, 20.0, 10.0));
    near(t.rect(k[1]), r(20.0, 10.0, 20.0, 30.0));
    near(t.rect(k[2]), r(40.0, 40.0, 20.0, 10.0));
}

#[test]
fn justify_center_and_end() {
    let (mut t, root, k) = tree_with(Style { justify: Justify::Center, ..Style::default() }, &[px(40.0, 10.0), px(20.0, 10.0)]);
    t.compute(root, Vec2::new(100.0, 10.0));
    near(t.rect(k[0]), r(20.0, 0.0, 40.0, 10.0));
    near(t.rect(k[1]), r(60.0, 0.0, 20.0, 10.0));
    t.style_mut(root).justify = Justify::End;
    t.compute(root, Vec2::new(100.0, 10.0));
    near(t.rect(k[0]), r(40.0, 0.0, 40.0, 10.0));
}

#[test]
fn auto_margins_center_and_win_over_justify() {
    // `margin: 0 auto` on a 40px box in a 100px row: 30 each side
    let m = Style { margin: Margin::horizontal_auto(), ..px(40.0, 10.0) };
    let (mut t, root, k) = tree_with(Style { justify: Justify::End, ..Style::default() }, &[m]);
    t.compute(root, Vec2::new(100.0, 10.0));
    near(t.rect(k[0]), r(30.0, 0.0, 40.0, 10.0));
}

#[test]
fn auto_margins_center_in_a_column_cross_axis() {
    // the page's #page: width 100% capped at max-width, margin 0 auto, in a column body
    let page = Style { w: Dim::Pct(100.0), max_w: Dim::Px(1480.0), margin: Margin::horizontal_auto(), grow: 1.0, ..Style::default() };
    let (mut t, root, k) = tree_with(Style { column: true, ..Style::default() }, &[page]);
    t.compute(root, Vec2::new(1600.0, 300.0));
    near(t.rect(k[0]), r(60.0, 0.0, 1480.0, 300.0));
}

#[test]
fn wrap_breaks_lines_and_uses_both_gaps() {
    // 3 boxes of 100x20 in 250px, gap 10 / 6: two fit (210), the third wraps; lines are 20 high
    let root = Style { wrap: true, gap: Vec2::new(10.0, 6.0), align_items: Align::Start, ..Style::default() };
    let (mut t, root, k) = tree_with(root, &[px(100.0, 20.0), px(100.0, 20.0), px(100.0, 20.0)]);
    t.compute(root, Vec2::new(250.0, 200.0));
    near(t.rect(k[0]), r(0.0, 0.0, 100.0, 20.0));
    near(t.rect(k[1]), r(110.0, 0.0, 100.0, 20.0));
    near(t.rect(k[2]), r(0.0, 26.0, 100.0, 20.0));
}

#[test]
fn wrap_container_sizes_itself_from_its_lines() {
    let root = Style { wrap: true, w: Dim::Px(250.0), ..Style::default() };
    let (mut t, outer, k) = tree_with(Style { column: true, align_items: Align::Start, ..Style::default() }, &[root]);
    for _ in 0..3 {
        let c = t.new_node(px(100.0, 20.0));
        t.append(k[0], c);
    }
    t.compute(outer, Vec2::new(400.0, 400.0));
    // two lines of 20 with no gap
    near(t.rect(k[0]), r(0.0, 0.0, 250.0, 40.0));
}

#[test]
fn order_reorders_without_moving_the_tree() {
    let (mut t, root, k) = tree_with(Style::default(), &[
        Style { order: 2, ..px(10.0, 10.0) },
        Style { order: 1, ..px(20.0, 10.0) },
        Style { order: 1, ..px(30.0, 10.0) },
    ]);
    t.compute(root, Vec2::new(100.0, 10.0));
    near(t.rect(k[1]), r(0.0, 0.0, 20.0, 10.0));
    near(t.rect(k[2]), r(20.0, 0.0, 30.0, 10.0)); // stable among equals
    near(t.rect(k[0]), r(50.0, 0.0, 10.0, 10.0));
}

#[test]
fn display_none_takes_no_room() {
    let gone = Style { display: Display::None, ..px(50.0, 50.0) };
    let (mut t, root, k) = tree_with(Style { gap: Vec2::new(8.0, 0.0), ..Style::default() }, &[px(10.0, 10.0), gone, px(10.0, 10.0)]);
    t.compute(root, Vec2::new(100.0, 10.0));
    near(t.rect(k[1]), r(0.0, 0.0, 0.0, 0.0));
    near(t.rect(k[2]), r(18.0, 0.0, 10.0, 10.0));
}

#[test]
fn baseline_aligns_texts_of_different_sizes() {
    // a 16px label (ascent 12, height 20) and a 10px number (ascent 8, height 14): the baselines meet
    let root = Style { align_items: Align::Baseline, ..Style::default() };
    let mut t = Tree::new();
    let root = t.new_node(root);
    let a = text(&mut t, 50.0, 50.0, 20.0, 12.0);
    let b = text(&mut t, 30.0, 30.0, 14.0, 8.0);
    t.append(root, a);
    t.append(root, b);
    t.compute(root, Vec2::new(200.0, 100.0));
    near(t.rect(a), r(0.0, 0.0, 50.0, 20.0));
    near(t.rect(b), r(50.0, 4.0, 30.0, 14.0));
}

#[test]
fn a_container_passes_up_its_first_childs_baseline() {
    // a column holding text (ascent 12) at the top: a row beside a text of ascent 8 aligns on it
    let mut t = Tree::new();
    let root = t.new_node(Style { align_items: Align::Baseline, ..Style::default() });
    let col = t.new_node(Style { column: true, pad: Edges::new(3.0, 0.0, 0.0, 0.0), ..Style::default() });
    let inner = text(&mut t, 40.0, 40.0, 20.0, 12.0);
    let other = text(&mut t, 30.0, 30.0, 14.0, 8.0);
    t.append(col, inner);
    t.append(root, col);
    t.append(root, other);
    t.compute(root, Vec2::new(200.0, 100.0));
    // col baseline = 3 + 12 = 15; other must sit 15 - 8 = 7 below the line start
    near(t.rect(other), r(40.0, 7.0, 30.0, 14.0));
}

#[test]
fn automatic_minimum_stops_text_at_its_longest_word() {
    // text 200 wide at most, 50 at least, in a 30px row: min-width:auto keeps it at 50
    let mut t = Tree::new();
    let root = t.new_node(Style::default());
    let a = text(&mut t, 200.0, 50.0, 10.0, 8.0);
    t.append(root, a);
    t.compute(root, Vec2::new(30.0, 40.0));
    assert_eq!(t.rect(a).w, 50.0);
    // the page's `min-width: 0` turns that off, and the text takes the 30 it is given
    t.style_mut(a).min_w = Dim::Px(0.0);
    t.compute(root, Vec2::new(30.0, 40.0));
    assert_eq!(t.rect(a).w, 30.0);
}

#[test]
fn text_shrinks_to_the_room_and_grows_in_height() {
    // 200 wide natural, room 100 (min-width auto = 50 allows it): wraps to 2 lines of 10
    let mut t = Tree::new();
    let root = t.new_node(Style { align_items: Align::Start, ..Style::default() });
    let a = text(&mut t, 200.0, 50.0, 10.0, 8.0);
    t.append(root, a);
    t.compute(root, Vec2::new(100.0, 100.0));
    near(t.rect(a), r(0.0, 0.0, 100.0, 20.0));
}

#[test]
fn a_nested_row_has_the_sum_of_its_minimums_as_its_own() {
    // the inner row holds two 40px-min texts (natural 100 each); in a 50px outer row it may not go under 80
    let mut t = Tree::new();
    let root = t.new_node(Style::default());
    let inner = t.new_node(Style::default());
    let a = text(&mut t, 100.0, 40.0, 10.0, 8.0);
    let b = text(&mut t, 100.0, 40.0, 10.0, 8.0);
    t.append(inner, a);
    t.append(inner, b);
    t.append(root, inner);
    t.compute(root, Vec2::new(50.0, 40.0));
    assert_eq!(t.rect(inner).w, 80.0);
}

#[test]
fn percentages_and_padding_use_the_inner_box() {
    // padding 10 leaves 180; a child of 50% is 90; with an unknown parent width it would be Auto
    let root = Style { pad: Edges::all(10.0), align_items: Align::Start, ..Style::default() };
    let (mut t, root, k) = tree_with(root, &[Style { w: Dim::Pct(50.0), h: Dim::Pct(100.0), ..Style::default() }]);
    t.compute(root, Vec2::new(200.0, 100.0));
    near(t.rect(k[0]), r(10.0, 10.0, 90.0, 80.0));
}

#[test]
fn aspect_ratio_fills_the_missing_axis() {
    let (mut t, root, k) = tree_with(Style { align_items: Align::Start, ..Style::default() }, &[Style {
        w: Dim::Px(80.0),
        aspect: Some(2.0),
        ..Style::default()
    }]);
    t.compute(root, Vec2::new(200.0, 100.0));
    near(t.rect(k[0]), r(0.0, 0.0, 80.0, 40.0));
}

#[test]
fn block_is_a_column_of_full_width_boxes() {
    let root = Style { display: Display::Block, gap: Vec2::new(9.0, 9.0), ..Style::default() };
    let (mut t, root, k) = tree_with(root, &[Style { h: Dim::Px(10.0), ..Style::default() }, Style { h: Dim::Px(12.0), ..Style::default() }]);
    t.compute(root, Vec2::new(70.0, 100.0));
    near(t.rect(k[0]), r(0.0, 0.0, 70.0, 10.0));
    near(t.rect(k[1]), r(0.0, 10.0, 70.0, 12.0)); // block ignores gap
}

#[test]
fn nested_rects_are_relative_and_abs_rect_adds_them_up() {
    let mut t = Tree::new();
    let root = t.new_node(Style { pad: Edges::all(10.0), ..Style::default() });
    let mid = t.new_node(Style { pad: Edges::all(5.0), ..px(100.0, 60.0) });
    let leaf = t.new_node(px(20.0, 20.0));
    t.append(root, mid);
    t.append(mid, leaf);
    t.compute(root, Vec2::new(300.0, 200.0));
    near(t.rect(mid), r(10.0, 10.0, 100.0, 60.0));
    near(t.rect(leaf), r(5.0, 5.0, 20.0, 20.0));
    near(t.abs_rect(leaf), r(15.0, 15.0, 20.0, 20.0));
}

#[test]
fn content_size_reports_how_far_children_reach() {
    let root = Style { column: true, ..Style::default() };
    let (mut t, root, _) = tree_with(root, &[
        Style { h: Dim::Px(300.0), shrink: 0.0, ..Style::default() },
        Style { h: Dim::Px(200.0), shrink: 0.0, ..Style::default() },
    ]);
    t.compute(root, Vec2::new(100.0, 100.0));
    assert_eq!(t.content_size(root), Vec2::new(100.0, 500.0));
}

#[test]
fn compute_twice_gives_the_same_rects() {
    let (mut t, root, k) = tree_with(Style::default(), &[px(10.0, 10.0), Style { grow: 1.0, ..Style::default() }]);
    t.compute(root, Vec2::new(100.0, 10.0));
    let first = t.rect(k[1]);
    t.compute(root, Vec2::new(100.0, 10.0));
    assert_eq!(first, t.rect(k[1]));
}

#[test]
fn dump_lists_names_and_rects_by_depth() {
    let mut t = Tree::new();
    let root = t.new_node(Style::default());
    let a = t.new_node(px(10.0, 5.5));
    t.set_name(root, "root");
    t.set_name(a, "a");
    t.append(root, a);
    t.compute(root, Vec2::new(40.0, 20.0));
    assert_eq!(t.dump(root), "root 0 0 40 20\n  a 0 0 10 5.5\n");
}
