//! Where the window was left: the text a placement file holds, read back at the next start, and the check
//! that keeps a window from reopening where no screen shows it. Pure, so it is tested without a window; the
//! calls to the system that fill and apply it are in `window.rs`.

/// What the system calls the normal placement: the rectangle the window has when it is neither maximised nor
/// minimised, in the screen's physical pixels (the work area's coordinates), and whether it is maximised.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub maximized: bool,
}

/// How much of the rectangle must lie on a screen for the window to be worth restoring there: enough to
/// grab its bar and drag it back. A corner of one pixel would "intersect" and leave a window nobody can find.
const MIN_SEEN_W: i32 = 64;
const MIN_SEEN_H: i32 = 32;

impl Placement {
    /// One line: `left top right bottom maximized`, the last as 0 or 1. Plain numbers, so a person can read it
    /// and a later version can add fields at the end.
    pub fn to_line(self) -> String {
        format!("{} {} {} {} {}\n", self.left, self.top, self.right, self.bottom, self.maximized as u8)
    }

    /// Reads the line back. Anything else (empty, torn, a rectangle with no area, a flag that is not 0 or 1)
    /// is `None`, and the window then opens where the system puts it. Fields past the fifth are ignored.
    pub fn parse(text: &str) -> Option<Placement> {
        let mut it = text.split_whitespace();
        let mut next = || it.next()?.parse::<i32>().ok();
        let (left, top, right, bottom) = (next()?, next()?, next()?, next()?);
        let maximized = match next()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        (right > left && bottom > top).then_some(Placement { left, top, right, bottom, maximized })
    }

    /// True when enough of the rectangle lies on one of the screens' work areas, each as
    /// `[left, top, right, bottom]`.
    pub fn is_seen_on(self, screens: &[[i32; 4]]) -> bool {
        screens.iter().any(|s| {
            let w = self.right.min(s[2]) - self.left.max(s[0]);
            let h = self.bottom.min(s[3]) - self.top.max(s[1]);
            // A rectangle smaller than the threshold needs only to be wholly inside.
            w >= MIN_SEEN_W.min(self.right - self.left) && h >= MIN_SEEN_H.min(self.bottom - self.top)
        })
    }

    /// The same rectangle grown to at least `min` (width, height), anchored at its top left. A saved size
    /// from an older, smaller minimum must not open a window the layout cannot fill.
    pub fn at_least(self, min: (i32, i32)) -> Placement {
        Placement { right: self.right.max(self.left + min.0), bottom: self.bottom.max(self.top + min.1), ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: Placement = Placement { left: 100, top: 50, right: 1380, bottom: 850, maximized: false };

    #[test]
    fn a_placement_reads_back_as_written() {
        assert_eq!(Placement::parse(&P.to_line()), Some(P));
        let m = Placement { maximized: true, left: -1920, ..P };
        assert_eq!(Placement::parse(&m.to_line()), Some(m), "a screen left of the main one has negative x");
    }

    #[test]
    fn the_line_is_five_plain_numbers() {
        assert_eq!(P.to_line(), "100 50 1380 850 0\n");
    }

    #[test]
    fn what_cannot_be_read_is_nothing() {
        for bad in ["", "   \n", "1 2 3", "1 2 3 4", "a b c d 0", "10 10 5 20 0", "10 10 20 10 0", "0 0 100 100 2", "0 0 100 100 -1", "0 0 1e3 100 0"] {
            assert_eq!(Placement::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_longer_line_from_a_later_version_still_reads() {
        assert_eq!(Placement::parse("100 50 1380 850 1 extra 7\n"), Some(Placement { maximized: true, ..P }));
    }

    #[test]
    fn a_window_is_restored_only_where_a_screen_shows_it() {
        let main = [0, 0, 1920, 1040];
        let right = [1920, 0, 3840, 1040];
        assert!(P.is_seen_on(&[main]));
        assert!(!P.is_seen_on(&[]));
        // The screen it was on is gone.
        let gone = Placement { left: 2000, right: 3280, ..P };
        assert!(!gone.is_seen_on(&[main]));
        assert!(gone.is_seen_on(&[main, right]));
        // Straddling two screens counts.
        let straddle = Placement { left: 1500, right: 2700, ..P };
        assert!(straddle.is_seen_on(&[main]));
        // Touching the edge, or a sliver, is not seen.
        assert!(!Placement { left: 1920, right: 3200, ..P }.is_seen_on(&[main]));
        assert!(!Placement { left: 1900, right: 3200, ..P }.is_seen_on(&[main]), "20 px is no handle");
        assert!(!Placement { top: 1030, bottom: 1830, ..P }.is_seen_on(&[main]), "10 px of height is no bar");
    }

    #[test]
    fn a_small_window_wholly_on_a_screen_is_seen() {
        let tiny = Placement { left: 10, top: 10, right: 40, bottom: 30, maximized: false };
        assert!(tiny.is_seen_on(&[[0, 0, 800, 600]]));
        assert!(!tiny.is_seen_on(&[[500, 500, 800, 600]]));
    }

    #[test]
    fn the_minimum_size_wins_over_a_smaller_saved_one() {
        let small = Placement { left: 100, top: 50, right: 600, bottom: 400, maximized: false };
        let grown = small.at_least((1000, 600));
        assert_eq!((grown.left, grown.top, grown.right, grown.bottom), (100, 50, 1100, 650));
        assert_eq!(P.at_least((1000, 600)), P, "a larger window is left alone");
        assert_eq!(P.at_least((0, 0)), P);
    }
}
