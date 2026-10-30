//! The accessibility tree: roles, names and focus order for what is on screen.
//!
//! # Why this is a projection, not a second bookkeeping system
//!
//! Every control the shell draws registers a hotspot, and the keyboard's focus
//! order is already `Hotspots::ids()` filtered by [`Hot::keyboard_focusable`].
//! The tree is built from exactly that table when an assistive technology (or a
//! test) asks for it, so it cannot drift from what is painted: a control that is
//! not drawn has no hotspot and therefore no node, and a node's rectangle is the
//! rectangle the pointer would hit.
//!
//! # Cost
//!
//! **Zero per frame.** Nothing is built while painting: the tree is assembled on
//! demand from the hotspot table the frame already produced, so an idle desktop
//! pays no allocation for an assistive technology that is not connected.
//!
//! # What a name is
//!
//! A name is the text the control already shows, or — for the glyph-only status
//! controls — the tooltip label that §70 added, which is why that tooltip is the
//! same string as the accessible name here. There is no invented label: a
//! control whose text this module cannot know is reported as unnamed rather than
//! given a generic name, so a page full of "Button" cannot look accessible.

use crate::canvas::Rect;
use crate::shell::Hot;

/// What a node is, in the vocabulary the tree is consumed with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A push button.
    Button,
    /// A run of text that is not a control.
    Text,
    /// A list container.
    List,
    /// One row of a list.
    ListItem,
    /// A modal dialog.
    Dialog,
    /// The status bar, or one control on it.
    StatusBar,
    /// A tab in a sidebar.
    Tab,
    /// One choice in a segmented picker.
    Segment,
    /// A grid container.
    Grid,
    /// One cell of a grid.
    GridCell,
    /// A window body.
    Window,
    /// A determinate or indeterminate progress control.
    ProgressBar,
}

impl Role {
    /// The role's name, as a tree dump prints it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Text => "text",
            Self::List => "list",
            Self::ListItem => "listitem",
            Self::Dialog => "dialog",
            Self::StatusBar => "statusbar",
            Self::Tab => "tab",
            Self::Segment => "segment",
            Self::Grid => "grid",
            Self::GridCell => "gridcell",
            Self::Window => "window",
            Self::ProgressBar => "progressbar",
        }
    }
}

/// One node of the tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// What it is.
    pub role: Role,
    /// The text it shows, or the tooltip that names it.
    pub name: String,
    /// Where it was drawn, in device pixels.
    pub rect: Rect,
    /// Whether the keyboard is on it. At most one node has this set.
    pub focused: bool,
    /// Whether the keyboard cannot reach it.
    pub disabled: bool,
    /// The tooltip label, when the control has one.
    pub tooltip: Option<&'static str>,
    /// The grid row, for a [`Role::GridCell`].
    pub row: Option<usize>,
    /// The grid column, for a [`Role::GridCell`].
    pub col: Option<usize>,
    /// A structural node (a bar, grid, dialog or window), not a control the
    /// keyboard stops on. Containers are in the tree so a reader knows where a
    /// group begins; they are not in the Tab order.
    pub container: bool,
}

/// A frame's accessibility tree.
#[derive(Debug, Clone, Default)]
pub struct Tree {
    /// The nodes, in paint order — which is also the Tab order.
    pub nodes: Vec<Node>,
    /// Visible hotspots this build could not name. Reported, never invented.
    pub unnamed: usize,
    /// The unnamed hotspots themselves, for a failure message or a report.
    pub unnamed_names: Vec<String>,
}

impl Tree {
    /// One line per node: `role "name" focused x,y wxh`.
    ///
    /// The dump is the regression test's snapshot: it is about roles, names and
    /// focus, not pixels, so it does not move when a colour token changes.
    #[must_use]
    pub fn dump(&self) -> String {
        let mut out = String::new();
        for node in &self.nodes {
            out.push_str(node.role.name());
            out.push_str(" \"");
            out.push_str(&node.name);
            out.push('"');
            if node.focused {
                out.push_str(" focused");
            }
            if node.disabled {
                out.push_str(" disabled");
            }
            out.push(' ');
            out.push_str(&format!(
                "{:.0},{:.0} {:.0}x{:.0}",
                node.rect.x, node.rect.y, node.rect.w, node.rect.h
            ));
            out.push('\n');
        }
        out
    }

    /// The focused node, if the keyboard is on a named control.
    #[must_use]
    pub fn focused(&self) -> Option<&Node> {
        self.nodes.iter().find(|node| node.focused)
    }

    /// The index of the focused node in paint order, which is the Tab order.
    #[must_use]
    pub fn focused_index(&self) -> Option<usize> {
        self.nodes.iter().position(|node| node.focused)
    }
}

/// The label a glyph-only control shows in its tooltip.
///
/// For these controls the tooltip *is* the name: there is no other text on
/// screen to read, which is the gap §70 closed and this batch makes visible to
/// assistive technology.
#[must_use]
pub fn control_tooltip(hot: Hot) -> Option<&'static str> {
    match hot {
        Hot::StatusNotifications => Some("Notifications"),
        Hot::StatusNetwork => Some("Network"),
        Hot::StatusAudio => Some("Sound"),
        Hot::StatusBattery => Some("Battery"),
        Hot::StatusPower => Some("Power mode"),
        Hot::StatusMetrics => Some("CPU and memory"),
        Hot::StatusClock => Some("Date and time"),
        Hot::StatusUser => Some("Account"),
        Hot::Logo => Some("LitricOS"),
        Hot::MenuApps => Some("Apps"),
        Hot::MenuWorkspace => Some("Workspace"),
        Hot::MenuEnvironment => Some("Environment"),
        Hot::DialogCancel => Some("Cancel"),
        Hot::LauncherSearch => Some("Search applications"),
        _ => None,
    }
}

pub(crate) fn container(role: Role, name: &str, rect: Rect) -> Node {
    Node {
        role,
        name: name.to_owned(),
        rect,
        focused: false,
        disabled: false,
        tooltip: None,
        row: None,
        col: None,
        container: true,
    }
}
