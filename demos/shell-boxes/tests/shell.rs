//! Step 7's checks on the world preview shell: the column widths of the three breakpoints, nothing
//! overlapping, and the debug pictures under `target/tmp`.

use nui_base::Rect;
use nui_layout::{NodeId, Tree};
use shell_boxes::{Columns, build, draw_boxes};

mod common;

const HEIGHT: f32 = 800.0;

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w - 0.01 && b.x < a.x + a.w - 0.01 && a.y < b.y + b.h - 0.01 && b.y < a.y + a.h - 0.01
}

/// Siblings never overlap and every child lies inside its parent, all the way down.
fn check_tree(t: &Tree, n: NodeId) {
    let kids = t.children(n);
    let parent = t.abs_rect(n);
    for (i, &a) in kids.iter().enumerate() {
        let ra = t.abs_rect(a);
        assert!(
            ra.x >= parent.x - 0.01 && ra.y >= parent.y - 0.01 && ra.x + ra.w <= parent.x + parent.w + 0.01 && ra.y + ra.h <= parent.y + parent.h + 0.01,
            "{} leaves {}: {ra:?} not in {parent:?}",
            t.name(a),
            t.name(n)
        );
        for &b in &kids[i + 1..] {
            assert!(!overlap(ra, t.abs_rect(b)), "{} overlaps {}", t.name(a), t.name(b));
        }
        check_tree(t, a);
    }
}

/// width, left, middle, right, page x. The middle is what is left of the page's inner width after the
/// two fixed columns and the two gaps:
///   1400: 1400 - 2*28 - 320 - 330 - 2*28 = 638
///   1200: 1200 - 2*22 - 296 - 280 - 2*24 = 532
///   1000: 1000 - 2*16 - 272 - 262 - 2*20 = 394
const CASES: [(f32, f32, f32, f32); 3] = [(1400.0, 320.0, 638.0, 330.0), (1200.0, 296.0, 532.0, 280.0), (1000.0, 272.0, 394.0, 262.0)];

#[test]
fn the_three_columns_have_the_pages_widths_at_each_breakpoint() {
    for (width, left, middle, right) in CASES {
        let s = build(width, HEIGHT);
        let c = Columns::for_width(width);
        let (l, m, r) = (s.tree.rect(s.left), s.tree.rect(s.middle), s.tree.rect(s.right));
        assert!(near(l.w, left) && near(m.w, middle) && near(r.w, right), "{width}: {} {} {}", l.w, m.w, r.w);
        // the columns follow each other with the page's gap, inside its padding
        assert!(near(l.x, c.pad.left) && near(m.x, l.x + l.w + c.gap) && near(r.x, m.x + m.w + c.gap));
        assert!(near(r.x + r.w + c.pad.right, width), "{width}: right edge");
        // and fill the page's height under the toolbar (10 + 28 = 38)
        let page = s.tree.rect(s.page);
        assert!(near(page.y, 38.0) && near(page.h, HEIGHT - 38.0), "{width}: page {page:?}");
        assert!(near(l.y, c.pad.top) && near(l.h, page.h - c.pad.top - c.pad.bottom));
        println!("{width}:\n{}", s.tree.dump(s.root));
    }
}

#[test]
fn nothing_overlaps_and_everything_stays_inside_its_parent() {
    for (width, ..) in CASES {
        let s = build(width, HEIGHT);
        check_tree(&s.tree, s.root);
    }
    check_tree(&build(1700.0, HEIGHT).tree, build(1700.0, HEIGHT).root);
}

#[test]
fn a_window_wider_than_1480_centres_the_page() {
    // #page max-width 1480 with margin 0 auto: 1600 leaves 60 on each side
    let s = build(1600.0, HEIGHT);
    let page = s.tree.rect(s.page);
    assert!(near(page.x, 60.0) && near(page.w, 1480.0), "{page:?}");
    assert!(near(s.tree.rect(s.left).w, 320.0));
}

#[test]
fn the_toolbar_is_centred_and_the_stage_leaves_room_for_under() {
    let s = build(1400.0, HEIGHT);
    let strip = s.tree.abs_rect(s.strip);
    assert!(near(strip.x + strip.w / 2.0, 700.0) && near(strip.y, 10.0) && near(strip.h, 28.0));
    let (stage, under) = (s.tree.rect(s.stage), s.tree.rect(s.under));
    assert!(near(under.h, 64.0) && near(stage.y + stage.h, under.y));
}

#[test]
fn debug_pictures_at_1400_1200_and_1000() {
    nui_base::log::add_sink(nui_base::log::stderr_sink(nui_base::log::Level::Info));
    for (width, ..) in CASES {
        let s = build(width, HEIGHT);
        let list = draw_boxes(&s.tree, s.root, width, HEIGHT);
        common::write_picture(&list, width as u32, HEIGHT as u32, &format!("shell_{width}.png"));
    }
}
