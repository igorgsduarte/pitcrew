//! The flag as PitCrew hands it over, mirrored from the WIT `flag-signal`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Disqualify,
    Black,
    Repair,
    Red,
    Checkered,
    White,
    Yellow,
    Blue,
    Green,
}

impl Flag {
    pub const ALL: [Flag; 9] = [
        Flag::Disqualify,
        Flag::Black,
        Flag::Repair,
        Flag::Red,
        Flag::Checkered,
        Flag::White,
        Flag::Yellow,
        Flag::Blue,
        Flag::Green,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Flag::Disqualify => "disqualify",
            Flag::Black => "black",
            Flag::Repair => "repair",
            Flag::Red => "red",
            Flag::Checkered => "checkered",
            Flag::White => "white",
            Flag::Yellow => "yellow",
            Flag::Blue => "blue",
            Flag::Green => "green",
        }
    }

    pub fn parse(s: &str) -> Option<Flag> {
        Flag::ALL.into_iter().find(|f| f.name() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartLight {
    Ready,
    Set,
    Go,
}

impl StartLight {
    pub fn name(self) -> &'static str {
        match self {
            StartLight::Ready => "ready",
            StartLight::Set => "set",
            StartLight::Go => "go",
        }
    }

    pub fn parse(s: &str) -> Option<StartLight> {
        [StartLight::Ready, StartLight::Set, StartLight::Go].into_iter().find(|l| l.name() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverFlag {
    Black,
    Repair,
    Disqualify,
}

impl DriverFlag {
    pub fn name(self) -> &'static str {
        match self {
            DriverFlag::Black => "black",
            DriverFlag::Repair => "repair",
            DriverFlag::Disqualify => "disqualify",
        }
    }

    pub fn parse(s: &str) -> Option<DriverFlag> {
        [DriverFlag::Black, DriverFlag::Repair, DriverFlag::Disqualify].into_iter().find(|d| d.name() == s)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Signal {
    pub primary: Option<Flag>,
    pub waving: bool,
    pub debris: bool,
    pub one_lap_to_green: bool,
    pub start: Option<StartLight>,
    pub driver: Vec<DriverFlag>,
}

impl Signal {
    pub fn describe(&self) -> String {
        let mut parts = vec![self.primary.map_or("none", Flag::name).to_string()];
        if self.waving {
            parts.push("waving".into());
        }
        if self.debris {
            parts.push("debris".into());
        }
        if self.one_lap_to_green {
            parts.push("one lap to green".into());
        }
        if let Some(s) = self.start {
            parts.push(format!("start {}", s.name()));
        }
        for d in &self.driver {
            parts.push(format!("driver {}", d.name()));
        }
        parts.join(" · ")
    }
}
