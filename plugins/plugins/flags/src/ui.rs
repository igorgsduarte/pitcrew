//! The declared screen, mirrored from the WIT `ui` interface so it can be built and tested natively.

use crate::host::StatusLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Muted,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Primary,
    Secondary,
    Destructive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Page { title: String, subtitle: Option<String> },
    Tabs { selected: String },
    Tab(String),
    Card { title: Option<String>, description: Option<String> },
    Grid { columns: u8, gap: Gap },
    Stack(Gap),
    Separator,
    Text { text: String, tone: Tone, mono: bool },
    Badge { text: String, level: StatusLevel },
    Alert { title: Option<String>, text: String, level: StatusLevel },
    Table { columns: Vec<String>, rows: Vec<Vec<String>>, selectable: bool },
    Progress { value: f32, label: Option<String> },
    Button { label: String, variant: Variant, icon: Option<String>, disabled: bool },
    Switch { label: String, checked: bool, disabled: bool },
    Slider { label: String, min: f64, max: f64, step: f64, value: f64 },
    Select { label: String, options: Vec<(String, String)>, value: String },
    Input { label: String, value: String, max_length: u32 },
    PixelGrid { width: u8, height: u8, palette: Vec<u32>, pixels: Vec<u8>, editable: bool },
    SettingsForm(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub parent: Option<String>,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Pressed,
    Toggled(bool),
    Number(f64),
    Text(String),
    Selected(String),
    Painted { x: u8, y: u8, color: u8 },
    Filled(u8),
    Palette(Vec<u32>),
}

#[derive(Debug, Default)]
pub struct Tree {
    pub nodes: Vec<Node>,
}

impl Tree {
    pub fn add(&mut self, id: impl Into<String>, parent: Option<&str>, kind: Kind) -> String {
        let id = id.into();
        self.nodes.push(Node { id: id.clone(), parent: parent.map(str::to_string), kind });
        id
    }

    pub fn text(&mut self, id: &str, parent: &str, text: impl Into<String>, tone: Tone, mono: bool) {
        self.add(id, Some(parent), Kind::Text { text: text.into(), tone, mono });
    }

    pub fn button(&mut self, id: &str, parent: &str, label: impl Into<String>, variant: Variant) {
        self.add(id, Some(parent), Kind::Button { label: label.into(), variant, icon: None, disabled: false });
    }

    pub fn find(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

/// The checks the host applies before drawing a tree. See docs/ui.md.
pub fn validate(nodes: &[Node]) -> Result<(), String> {
    use std::collections::HashSet;
    if nodes.len() > 500 {
        return Err(format!("{} nodes; the limit is 500", nodes.len()));
    }
    let roots: Vec<_> = nodes.iter().filter(|n| n.parent.is_none()).collect();
    if roots.len() != 1 || !matches!(roots[0].kind, Kind::Page { .. }) {
        return Err("exactly one root, and it must be a page".into());
    }
    let mut seen = HashSet::new();
    for n in nodes {
        if !seen.insert(n.id.as_str()) {
            return Err(format!("duplicate id \"{}\"", n.id));
        }
        if let Some(p) = &n.parent {
            if !seen.contains(p.as_str()) {
                return Err(format!("\"{}\" names parent \"{p}\" before it exists", n.id));
            }
        }
        match &n.kind {
            Kind::Grid { columns, .. } if !(1..=3).contains(columns) => return Err(format!("\"{}\": grid columns must be 1 to 3", n.id)),
            Kind::Table { rows, .. } if rows.len() > 200 => return Err(format!("\"{}\": more than 200 rows", n.id)),
            Kind::PixelGrid { width, height, palette, pixels, .. } => {
                if !(1..=32).contains(width) || !(1..=32).contains(height) {
                    return Err(format!("\"{}\": pixel grid must be 1 to 32 a side", n.id));
                }
                if palette.len() > 16 || pixels.len() != *width as usize * *height as usize {
                    return Err(format!("\"{}\": palette ≤ 16 and pixels = width × height", n.id));
                }
            }
            Kind::Tab(_) => {
                let parent_is_tabs = n.parent.as_ref().and_then(|p| nodes.iter().find(|m| &m.id == p)).is_some_and(|m| matches!(m.kind, Kind::Tabs { .. }));
                if !parent_is_tabs {
                    return Err(format!("\"{}\": a tab must be a child of tabs", n.id));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
