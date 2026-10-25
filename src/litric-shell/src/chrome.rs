//! Geometry and surface constants of the shell's chrome: the top bar, the dock
//! and the lock screen's composition.
//!
//! One table, so a number that two views must agree on (the bar's height is
//! used by the window manager, the damage planner and the bar itself) has one
//! home, and no view carries a private copy of it. Every value is in logical
//! pixels; [`crate::ui::Tokens::px`] turns it into device pixels.

/// The top bar's height. Apple's menu bar is 24 pt; four more logical pixels
/// give the pointer a target it can hit without aiming.
pub const TOP_BAR_H: f32 = 28.0;

/// How opaque the top bar's surface is. A flat translucent fill, not a blur:
/// the bar is redrawn whenever the clock turns, and a blurred strip across the
/// whole width is the most expensive way to make text readable. 78 % keeps the
/// wallpaper present and holds the text's contrast the same on every
/// wallpaper.
pub const TOP_BAR_SURFACE_ALPHA: f32 = 0.78;

/// The bar's own side padding.
pub const TOP_BAR_PAD: f32 = 14.0;

/// The size a status icon is drawn at.
pub const TOP_BAR_ICON: f32 = 16.0;

/// The distance between the left edges of two neighbouring status icons.
pub const TOP_BAR_STEP: f32 = 26.0;

/// The distance between the mark and the application name, and between a menu
/// and the next one.
pub const TOP_BAR_GAP: f32 = 18.0;

/// The diameter of a workspace pip.
pub const TOP_BAR_PIP: f32 = 6.0;

/// The width, in logical pixels, below which the bar leaves out the date. The
/// supported minimum display is 960 wide; the date is the first thing to go.
pub const TOP_BAR_DATE_MIN_W: f32 = 1100.0;

/// The dock's height at rest. Low and wide: it holds the applications and
/// nothing taller than they are.
pub const DOCK_H: f32 = 62.0;

/// The side of an application tile in the dock.
pub const DOCK_ICON: f32 = 46.0;

/// The distance between two tiles.
pub const DOCK_GAP: f32 = 6.0;

/// The dock's inner padding at its ends.
pub const DOCK_PAD: f32 = 12.0;

/// How far a hovered tile rises. The strip has no room for magnification, and
/// re-cutting every icon on every pointer move is work the frame cannot afford;
/// a fixed lift costs one tile.
pub const DOCK_LIFT: f32 = 7.0;

/// The gap between the dock and the bottom edge of the screen.
pub const DOCK_FLOAT: f32 = 12.0;

/// The corner radius of the resting dock.
pub const DOCK_RADIUS: f32 = 22.0;

/// The width of the search field at rest.
pub const DOCK_SEARCH_W: f32 = 150.0;

/// The room a separator takes between two groups of tiles.
pub const DOCK_SEPARATOR: f32 = 12.0;

/// A tile's corner radius as a fraction of its side: a squircle, not a circle.
pub const DOCK_TILE_RADIUS: f32 = 0.28;

/// The lock screen's avatar diameter.
pub const LOCK_AVATAR: f32 = 72.0;

/// The password field's width and height. A pill, not a card: the field is one
/// line of text and one button, and it should look like it.
pub const LOCK_FIELD_W: f32 = 280.0;
pub const LOCK_FIELD_H: f32 = 40.0;

/// The distance from the screen's bottom edge to the field's bottom edge.
pub const LOCK_FIELD_FROM_BOTTOM: f32 = 136.0;

/// A widget chip's size on the lock screen.
pub const LOCK_CHIP_W: f32 = 212.0;
pub const LOCK_CHIP_H: f32 = 60.0;

/// The room between the clock's baseline and the widget row.
pub const LOCK_CLOCK_TO_CHIPS: f32 = 44.0;


/// Pure white, for a hairline of light on glass and for a mark under the pointer.
pub const WHITE: u32 = 0xFF_FF_FF_FF;
