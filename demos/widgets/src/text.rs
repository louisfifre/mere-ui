//! The demo's catalog: the page's wording (preview.html line numbers in the comments).

/// Line 462.
pub const GENERATE: &str = "Generate preview";
/// Line 464, `detailValue(16)`.
pub const DETAIL: &str = "16 blocks";
/// Line 480.
pub const EXPORT: &str = "Export";
/// Line 554.
pub const ISLANDS: &str = "Islands";
/// Lines 528-529.
pub const ON: &str = "on";
pub const OFF: &str = "off";
/// Line 456.
pub const SEED: &str = "Seed";
/// Line 467.
pub const UP_TO_DATE: &str = "The map shows these settings.";
/// Line 490.
pub const EXPORT_FAILED: &str = "The export failed.";

/// Line 455: the fold's title, uppercased by the widget as the page's CSS does.
pub const WORLD: &str = "World";
/// Line 581.
pub const SETTINGS: &str = "Settings";
/// Line 492.
pub const HISTORY: &str = "History";
/// Line 588.
pub const VIEWS: [&str; 5] = ["Biomes", "Flat", "Relief", "Temperature", "Structures"];
/// Line 619.
pub const SECTION: &str = "Section";
/// Line 589.
pub const SHAPES: [&str; 2] = ["2D", "3D"];
/// Line 582.
pub const PACK: &str = "Pack";
/// Line 587, `profileDefault(n)`.
pub const TRY: [&str; 3] = ["Try 2", "Try 3", "Try 4"];
/// Lines 464 (`detailValue`) and 461; the waits are PLACEHOLDER.
pub const DETAILS: [(&str, &str); 3] = [("64 blocks", "10 s"), ("32 blocks", "25 s"), ("16 blocks", "1 min 30 s")];
/// Line 538.
pub const VERTICAL_SCALE: &str = "Vertical scale";
/// Line 547.
pub const BOTTOM: &str = "Bottom of the world";
/// Line 550.
pub const LAND: &str = "Land against ocean";
/// Line 543.
pub const ROLLING: &str = "Rolling hills";
/// PLACEHOLDER: demo wording, not the page's.
pub const NO_RUN: &str = "No run yet";

pub const CAP_STRIP: &str = "strip: Flat and 2D pressed, pointer on Temperature";
pub const CAP_COARSE: &str = "strip on a coarse map: three disabled, pointer on the pressed Biomes";
pub const CAP_CHOICE: &str = "choice: Pack pressed, pointer on Try 3, Try 4 disabled";
pub const CAP_DETAILS: &str = "details: 32 pressed, 16 disabled";
pub const CAP_FOLD: &str = "fold closed";
pub const CAP_FOLD_HOVER: &str = "fold closed, pointer on it";
pub const CAP_FOLD_OPEN: &str = "fold open: a choice, number fields and a switch";

pub const NORMAL: &str = "normal";
pub const HOVER: &str = "hover";
pub const HELD: &str = "held";
pub const DISABLED: &str = "disabled";

pub const WINDOW_TITLE: &str = "Widgets";
