//! The height bands of the relief view: their names and colours, ported from preview.html so that the list
//! shows what the page's does (lines 509, 1040-1045, 1091-1102, 1134, 1137, 1353-1355).

/// `seaLevel` of `C:\Games\preview\out\preview.json`.
pub const SEA: f64 = 63.0;
/// Line 1134: where one band of height ends and the next starts, above the sea.
pub const HEIGHT_STEPS: [f64; 4] = [100.0, 150.0, 200.0, 250.0];
/// Line 1137: the same under the sea, in blocks of water over the floor.
pub const SEA_DEPTHS: [f64; 2] = [20.0, 45.0];

/// Lines 1040-1045. `Math.round` is `(x + 0.5).floor()`; the `%` of floats is JS's too.
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> [u8; 3] {
    let (s, l) = (s / 100.0, l / 100.0);
    let k = |n: f64| (n + h / 30.0) % 12.0;
    let a = s * l.min(1.0 - l);
    let f = |n: f64| {
        let kn = k(n);
        let v = l - a * (-1.0f64).max((kn - 3.0).min((9.0 - kn).min(1.0)));
        (255.0 * v + 0.5).floor() as u8
    };
    [f(0.0), f(8.0), f(4.0)]
}

/// Lines 1091-1102: lowland green, then ochre, rock, snow; below the sea, deeper is darker.
pub fn height_colour(y: f64, sea: f64) -> [u8; 3] {
    if y < sea {
        let d = ((sea - y) / 60.0).min(1.0);
        return hsl_to_rgb(212.0, 55.0, 46.0 - d * 26.0);
    }
    let stops: [(f64, [f64; 3]); 6] = [
        (0.0, [110.0, 40.0, 42.0]),
        (40.0, [80.0, 40.0, 50.0]),
        (90.0, [40.0, 38.0, 48.0]),
        (150.0, [25.0, 20.0, 45.0]),
        (210.0, [0.0, 0.0, 70.0]),
        (260.0, [0.0, 0.0, 96.0]),
    ];
    let up = y - sea;
    for i in 1..stops.len() {
        if up <= stops[i].0 {
            let (a, ca) = stops[i - 1];
            let (b, cb) = stops[i];
            let t = (up - a) / (b - a);
            return hsl_to_rgb(ca[0] + (cb[0] - ca[0]) * t, ca[1] + (cb[1] - ca[1]) * t, ca[2] + (cb[2] - ca[2]) * t);
        }
    }
    [245, 245, 245]
}

/// Line 509.
pub fn band_names(sea: f64, steps: &[f64], depths: &[f64]) -> Vec<String> {
    let deepest = depths[depths.len() - 1];
    let mut out = vec![format!("Sea, deeper than {deepest}")];
    let shelves: Vec<String> =
        depths.iter().enumerate().map(|(i, deep)| format!("Sea, {} to {deep} deep", if i > 0 { depths[i - 1] } else { 0.0 })).collect();
    out.extend(shelves.into_iter().rev());
    for (i, top) in steps.iter().enumerate() {
        out.push(format!("{} to {top}", if i > 0 { steps[i - 1] } else { sea }));
    }
    out.push(format!("Above {}", steps[steps.len() - 1]));
    out
}

/// Lines 1353-1355: each band takes the colour of its middle.
pub fn band_colours(sea: f64, steps: &[f64], depths: &[f64]) -> Vec<[u8; 3]> {
    let mut out = vec![height_colour(sea - depths[depths.len() - 1] - 15.0, sea)];
    let shelves: Vec<[u8; 3]> = depths
        .iter()
        .enumerate()
        .map(|(i, deep)| height_colour(sea - ((if i > 0 { depths[i - 1] } else { 0.0 }) + deep) / 2.0, sea))
        .collect();
    out.extend(shelves.into_iter().rev());
    let lows: Vec<f64> = std::iter::once(sea).chain(steps.iter().copied()).collect();
    for (i, low) in lows.iter().enumerate() {
        out.push(height_colour(if i < steps.len() { (low + steps[i]) / 2.0 } else { low + 30.0 }, sea));
    }
    out
}

/// PLACEHOLDER: the typical width and the share of each band. Step 18 computes them from the run.
pub const BAND_FIGURES: [(&str, &str); 8] = [
    ("900 blocks", "14.2%"),
    ("650 blocks", "18.6%"),
    ("300 blocks", "9.1%"),
    ("1,150 blocks", "27.4%"),
    ("700 blocks", "17.8%"),
    ("400 blocks", "8.3%"),
    ("200 blocks", "3.5%"),
    ("100 blocks", "1.1%"),
];
