//! The page's one transition: the tools' options sliding open and shut in 220 ms with CSS `ease`
//! (launcher line 124, `grid-template-columns 220ms ease`).

/// Line 124: 220 ms.
pub const SLIDE_SECONDS: f64 = 0.22;

/// CSS `ease`: cubic-bezier(0.25, 0.1, 0.25, 1.0). The curve is parametric in its own variable; the time `t`
/// is its x, so x is solved for the parameter (Newton's steps, then bisection where the slope is too flat),
/// and y at that parameter is the progress.
pub fn ease(t: f32) -> f32 {
    const X1: f64 = 0.25;
    const Y1: f64 = 0.1;
    const X2: f64 = 0.25;
    const Y2: f64 = 1.0;
    let x = f64::from(t.clamp(0.0, 1.0));
    let bez = |a: f64, b: f64, u: f64| 3.0 * a * (1.0 - u) * (1.0 - u) * u + 3.0 * b * (1.0 - u) * u * u + u * u * u;
    let slope = |a: f64, b: f64, u: f64| 3.0 * a * (1.0 - u) * (1.0 - u) + 6.0 * (b - a) * (1.0 - u) * u + 3.0 * (1.0 - b) * u * u;
    let mut u = x;
    let mut solved = false;
    for _ in 0..8 {
        let err = bez(X1, X2, u) - x;
        if err.abs() < 1e-7 {
            solved = true;
            break;
        }
        let d = slope(X1, X2, u);
        if d.abs() < 1e-6 {
            break;
        }
        u -= err / d;
    }
    if !solved || !(0.0..=1.0).contains(&u) {
        let (mut lo, mut hi) = (0.0f64, 1.0f64);
        u = x;
        for _ in 0..60 {
            if bez(X1, X2, u) < x {
                lo = u;
            } else {
                hi = u;
            }
            u = (lo + hi) / 2.0;
        }
    }
    bez(Y1, Y2, u) as f32
}

/// A value moving between 0 and 1 (closed and open) over `SLIDE_SECONDS`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Slide {
    pub from: f32,
    pub to: f32,
    /// The application's clock when it started.
    pub since: f64,
}

impl Slide {
    /// At rest, open or shut.
    pub fn settled(open: bool) -> Slide {
        let v = if open { 1.0 } else { 0.0 };
        Slide { from: v, to: v, since: f64::NEG_INFINITY }
    }

    fn progress(&self, now: f64) -> f32 {
        ((now - self.since) / SLIDE_SECONDS).clamp(0.0, 1.0) as f32
    }

    /// Where it stands at `now`. With `reduced` (the system asks for no animation) it is at its end at once.
    pub fn value(&self, now: f64, reduced: bool) -> f32 {
        if reduced {
            return self.to;
        }
        self.from + (self.to - self.from) * ease(self.progress(now))
    }

    /// True while it still moves: frames are asked for only then.
    pub fn moving(&self, now: f64, reduced: bool) -> bool {
        !reduced && self.from != self.to && now - self.since < SLIDE_SECONDS
    }

    /// Heads for open or shut. Started from where it stands, with the whole 220 ms again: an INTERPRETATION
    /// of how Chrome reverses a transition (it shortens the new one by the progress already made; the
    /// difference is not seen in 220 ms).
    pub fn toward(&mut self, open: bool, now: f64, reduced: bool) {
        let to = if open { 1.0 } else { 0.0 };
        if to == self.to {
            return;
        }
        *self = Slide { from: self.value(now, reduced), to, since: now };
    }
}
