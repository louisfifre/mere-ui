//! The Relief texture: the page's `paint()` for its height view, point for point. Each sample gets a colour
//! from its height above the sea, lit from the north-west by the slope towards its neighbours.

use crate::world::World;

/// The page's `hslToRgb`: hue in degrees, saturation and lightness in percent, channels rounded to 0..255.
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> [f64; 3] {
    let (s, l) = (s / 100.0, l / 100.0);
    let k = |n: f64| (n + h / 30.0) % 12.0;
    let a = s * l.min(1.0 - l);
    let f = |n: f64| (255.0 * (l - a * (-1.0f64).max((k(n) - 3.0).min((9.0 - k(n)).min(1.0))))).round();
    [f(0.0), f(8.0), f(4.0)]
}

/// Height above the sea: lowland green, then ochre, rock, snow. Below: deeper is darker.
pub fn height_colour(y: f64, sea: f64) -> [f64; 3] {
    if y < sea {
        let d = ((sea - y) / 60.0).min(1.0);
        return hsl_to_rgb(212.0, 55.0, 46.0 - d * 26.0);
    }
    // [height above the sea, [hue, saturation, lightness]]
    const STOPS: [(f64, [f64; 3]); 6] = [
        (0.0, [110.0, 40.0, 42.0]),
        (40.0, [80.0, 40.0, 50.0]),
        (90.0, [40.0, 38.0, 48.0]),
        (150.0, [25.0, 20.0, 45.0]),
        (210.0, [0.0, 0.0, 70.0]),
        (260.0, [0.0, 0.0, 96.0]),
    ];
    let up = y - sea;
    for i in 1..STOPS.len() {
        if up <= STOPS[i].0 {
            let (a, ca) = STOPS[i - 1];
            let (b, cb) = STOPS[i];
            let t = (up - a) / (b - a);
            let mix = |k: usize| ca[k] + (cb[k] - ca[k]) * t;
            return hsl_to_rgb(mix(0), mix(1), mix(2));
        }
    }
    [245.0, 245.0, 245.0]
}

/// RGBA rows, `samples` a side. Water is lit like land here, as in the page's Relief view, where the sea
/// floor is ground like the rest.
pub fn paint(world: &World) -> Vec<u8> {
    let n = world.samples;
    let h = &world.heights;
    let sea = world.sea_level as f64;
    let mut rgba = vec![255u8; n * n * 4];
    for i in 0..n * n {
        let x = i % n;
        let y = h[i] as f64;
        // At an edge the missing neighbour is the sample itself, so the slope there is one-sided.
        let west = if x > 0 { h[i - 1] as f64 } else { y };
        let east = if x < n - 1 { h[i + 1] as f64 } else { y };
        let north = if i >= n { h[i - n] as f64 } else { y };
        let south = if i < n * n - n { h[i + n] as f64 } else { y };
        let slope = (east - west) + (south - north);
        let shade = (1.0 + slope * 0.012).clamp(0.75, 1.25);
        let c = height_colour(y, sea);
        for k in 0..3 {
            // A canvas pixel array rounds half to even and clamps; the same here.
            rgba[i * 4 + k] = (c[k] * shade).round_ties_even().clamp(0.0, 255.0) as u8;
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_matches_known_colours() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 100.0), [255.0, 255.0, 255.0]);
        assert_eq!(hsl_to_rgb(0.0, 0.0, 0.0), [0.0, 0.0, 0.0]);
        assert_eq!(hsl_to_rgb(0.0, 100.0, 50.0), [255.0, 0.0, 0.0]);
        assert_eq!(hsl_to_rgb(120.0, 100.0, 50.0), [0.0, 255.0, 0.0]);
        assert_eq!(hsl_to_rgb(240.0, 100.0, 50.0), [0.0, 0.0, 255.0]);
    }

    #[test]
    fn height_colour_follows_the_stops() {
        // At the sea level the first stop; at its last stop the white-ish top; beyond it the fixed colour.
        assert_eq!(height_colour(63.0, 63.0), hsl_to_rgb(110.0, 40.0, 42.0));
        assert_eq!(height_colour(63.0 + 260.0, 63.0), hsl_to_rgb(0.0, 0.0, 96.0));
        assert_eq!(height_colour(400.0, 63.0), [245.0, 245.0, 245.0]);
        // Halfway between the first two stops.
        assert_eq!(height_colour(63.0 + 20.0, 63.0), hsl_to_rgb(95.0, 40.0, 46.0));
        // Deeper water is darker, and stops getting darker 60 blocks down.
        let shallow = height_colour(62.0, 63.0);
        let deep = height_colour(3.0, 63.0);
        assert!(deep[1] < shallow[1]);
        assert_eq!(height_colour(-100.0, 63.0), deep);
    }

    #[test]
    fn flat_ground_is_not_shaded_and_a_slope_is() {
        let world = |heights: Vec<i16>| World {
            seed: String::new(),
            samples: 3,
            step: 16.0,
            sea_level: 63,
            heights,
            biome_count: 1,
            biome_total: None,
        };
        let flat = paint(&world(vec![80; 9]));
        let c = height_colour(80.0, 63.0);
        assert_eq!(&flat[16..19], &[c[0] as u8, c[1] as u8, c[2] as u8]);
        // Rising to the east and south makes the centre brighter than flat... lit from the north-west means
        // a positive slope (east - west) brightens.
        let ramp = paint(&world(vec![80, 80, 80, 80, 80, 90, 80, 90, 90]));
        assert_ne!(&ramp[16..19], &flat[16..19]);
        assert_eq!(flat[19], 255);
    }
}
