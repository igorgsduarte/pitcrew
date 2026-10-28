//! Animation packs: `pack.json`, `animations/<id>.json` and `mapping.json`. See docs/content-packs.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::signal::{DriverFlag, Flag, Signal, StartLight};

pub const CONTENT_TYPE: &str = "flags.animations";
pub const BUILTIN_PACK: &str = "pitcrew.flags.classic";
pub const SIZE: usize = 8;
pub const MAX_COLORS: usize = 16;
pub const MAX_FPS: u32 = 30;
pub const MAX_FRAMES: usize = 120;
pub const OFF: char = '.';

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PackFile {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub version: String,
    pub animations: Vec<String>,
    #[serde(default)]
    pub has_mapping: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationFile {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub id: String,
    /// One character → "RRGGBB". `.` must be present and means off.
    pub palette: BTreeMap<String, String>,
    pub fps: u32,
    #[serde(rename = "loop")]
    pub looped: bool,
    /// Each frame is 8 strings of 8 palette characters, top row first.
    pub frames: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub id: String,
    /// The palette characters; index 0 is always `.` (off).
    pub letters: Vec<char>,
    pub colors: Vec<u32>,
    pub fps: u32,
    pub looped: bool,
    pub frames: Vec<[u8; 64]>,
}

impl Animation {
    pub fn compile(file: &AnimationFile) -> Result<Animation, String> {
        let id = &file.id;
        if !file.palette.contains_key(".") {
            return Err(format!("{id}: the palette needs \".\" (off)"));
        }
        if file.palette.len() > MAX_COLORS {
            return Err(format!("{id}: at most {MAX_COLORS} colours"));
        }
        let mut letters = vec![OFF];
        let mut colors = vec![parse_hex(id, &file.palette["."])?];
        for (key, hex) in &file.palette {
            let mut chars = key.chars();
            let (Some(c), None) = (chars.next(), chars.next()) else {
                return Err(format!("{id}: palette key \"{key}\" must be one character"));
            };
            if c == OFF {
                continue;
            }
            letters.push(c);
            colors.push(parse_hex(id, hex)?);
        }
        if !(1..=MAX_FPS).contains(&file.fps) {
            return Err(format!("{id}: fps must be 1 to {MAX_FPS}"));
        }
        if file.frames.is_empty() || file.frames.len() > MAX_FRAMES {
            return Err(format!("{id}: 1 to {MAX_FRAMES} frames"));
        }
        let mut frames = Vec::with_capacity(file.frames.len());
        for (n, rows) in file.frames.iter().enumerate() {
            if rows.len() != SIZE {
                return Err(format!("{id}: frame {} has {} rows, not {SIZE}", n + 1, rows.len()));
            }
            let mut px = [0u8; 64];
            for (y, row) in rows.iter().enumerate() {
                let chars: Vec<char> = row.chars().collect();
                if chars.len() != SIZE {
                    return Err(format!("{id}: frame {} row {} is not {SIZE} characters", n + 1, y + 1));
                }
                for (x, c) in chars.into_iter().enumerate() {
                    let Some(i) = letters.iter().position(|l| *l == c) else {
                        return Err(format!("{id}: frame {} row {} uses \"{c}\", which is not in the palette", n + 1, y + 1));
                    };
                    px[y * SIZE + x] = i as u8;
                }
            }
            frames.push(px);
        }
        Ok(Animation { id: id.clone(), letters, colors, fps: file.fps, looped: file.looped, frames })
    }

    pub fn to_file(&self) -> AnimationFile {
        let palette = self
            .letters
            .iter()
            .zip(&self.colors)
            .map(|(l, c)| (l.to_string(), format!("{c:06X}")))
            .collect();
        let frames = self
            .frames
            .iter()
            .map(|px| {
                px.chunks(SIZE)
                    .map(|row| row.iter().map(|&i| self.letters[i as usize]).collect::<String>())
                    .collect()
            })
            .collect();
        AnimationFile { schema: None, id: self.id.clone(), palette, fps: self.fps, looped: self.looped, frames }
    }

    /// The frame shown `elapsed_ms` after the animation started.
    pub fn frame_at(&self, elapsed_ms: u64) -> &[u8; 64] {
        let step = (elapsed_ms * self.fps as u64 / 1000) as usize;
        let i = if self.looped { step % self.frames.len() } else { step.min(self.frames.len() - 1) };
        &self.frames[i]
    }

    pub fn blank() -> Animation {
        Animation {
            id: "idle".into(),
            letters: vec![OFF],
            colors: vec![0],
            fps: 1,
            looped: true,
            frames: vec![[0u8; 64]],
        }
    }
}

fn parse_hex(id: &str, hex: &str) -> Result<u32, String> {
    let ok = hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit());
    if !ok {
        return Err(format!("{id}: colour \"{hex}\" must be RRGGBB"));
    }
    u32::from_str_radix(hex, 16).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WhenFile {
    pub primary: Option<String>,
    pub waving: Option<bool>,
    pub debris: Option<bool>,
    pub one_lap_to_green: Option<bool>,
    pub start: Option<String>,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum HoldFile {
    Seconds(f64),
    Setting(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RuleFile {
    pub when: WhenFile,
    pub show: String,
    pub hold_s: Option<HoldFile>,
    pub then: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct When {
    pub primary: Option<Flag>,
    pub waving: Option<bool>,
    pub debris: Option<bool>,
    pub one_lap_to_green: Option<bool>,
    pub start: Option<StartLight>,
    pub driver: Option<DriverFlag>,
}

impl When {
    pub fn matches(&self, s: &Signal) -> bool {
        self.primary.is_none_or(|p| s.primary == Some(p))
            && self.waving.is_none_or(|w| s.waving == w)
            && self.debris.is_none_or(|d| s.debris == d)
            && self.one_lap_to_green.is_none_or(|o| s.one_lap_to_green == o)
            && self.start.is_none_or(|l| s.start == Some(l))
            && self.driver.is_none_or(|d| s.driver.contains(&d))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hold {
    Seconds(f64),
    GreenHoldSetting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub when: When,
    pub show: String,
    pub hold: Option<Hold>,
    pub then: Option<String>,
}

pub fn compile_mapping(rules: &[RuleFile]) -> Result<Vec<Rule>, String> {
    rules
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let n = i + 1;
            let w = &r.when;
            let when = When {
                primary: w.primary.as_deref().map(|p| Flag::parse(p).ok_or(format!("rule {n}: unknown flag \"{p}\""))).transpose()?,
                waving: w.waving,
                debris: w.debris,
                one_lap_to_green: w.one_lap_to_green,
                start: w.start.as_deref().map(|s| StartLight::parse(s).ok_or(format!("rule {n}: unknown start light \"{s}\""))).transpose()?,
                driver: w.driver.as_deref().map(|d| DriverFlag::parse(d).ok_or(format!("rule {n}: unknown driver flag \"{d}\""))).transpose()?,
            };
            let hold = match &r.hold_s {
                None => None,
                Some(HoldFile::Seconds(s)) if *s >= 0.0 => Some(Hold::Seconds(*s)),
                Some(HoldFile::Seconds(_)) => return Err(format!("rule {n}: holdS cannot be negative")),
                Some(HoldFile::Setting(s)) if s == "setting:greenHoldS" => Some(Hold::GreenHoldSetting),
                Some(HoldFile::Setting(s)) => return Err(format!("rule {n}: holdS \"{s}\" is not a number or \"setting:greenHoldS\"")),
            };
            Ok(Rule { when, show: r.show.clone(), hold, then: r.then.clone() })
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub animations: BTreeMap<String, Animation>,
    pub mapping: Vec<Rule>,
}

impl Pack {
    /// Reads a pack through `read(path)`. A pack without a mapping keeps `fallback`'s.
    pub fn load(
        read: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
        fallback: Option<&Pack>,
    ) -> Result<Pack, String> {
        let file: PackFile = json(&read("pack.json")?, "pack.json")?;
        if file.content_type != CONTENT_TYPE {
            return Err(format!("pack.json: type is \"{}\", expected \"{CONTENT_TYPE}\"", file.content_type));
        }
        let mut animations = fallback.map(|f| f.animations.clone()).unwrap_or_default();
        for name in &file.animations {
            let path = format!("animations/{name}.json");
            let a: AnimationFile = json(&read(&path)?, &path)?;
            if &a.id != name {
                return Err(format!("{path}: id is \"{}\", expected \"{name}\"", a.id));
            }
            animations.insert(name.clone(), Animation::compile(&a)?);
        }
        let mapping = if file.has_mapping {
            let rules: Vec<RuleFile> = json(&read("mapping.json")?, "mapping.json")?;
            compile_mapping(&rules)?
        } else if let Some(f) = fallback {
            f.mapping.clone()
        } else {
            return Err("pack.json: the base pack needs a mapping (hasMapping: true)".into());
        };
        for r in &mapping {
            for name in std::iter::once(&r.show).chain(r.then.iter()) {
                if !animations.contains_key(name) {
                    return Err(format!("mapping.json: \"{name}\" is not an animation in this pack"));
                }
            }
        }
        Ok(Pack { id: file.id, name: file.name, animations, mapping })
    }

    pub fn get(&self, name: &str) -> Option<&Animation> {
        self.animations.get(name)
    }
}

fn json<T: for<'de> Deserialize<'de>>(bytes: &[u8], what: &str) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("{what}: {e}"))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::PathBuf;

    pub fn classic_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content").join("classic")
    }

    pub fn classic() -> Pack {
        let dir = classic_dir();
        let mut read = |p: &str| std::fs::read(dir.join(p)).map_err(|e| format!("{p}: {e}"));
        Pack::load(&mut read, None).expect("the classic pack loads")
    }

    #[test]
    fn the_classic_pack_loads_and_every_rule_names_an_animation() {
        let p = classic();
        assert_eq!(p.id, BUILTIN_PACK);
        assert!(p.get("idle").is_some());
        assert!(p.mapping.last().unwrap().when == When::default(), "the last rule catches everything");
    }

    #[test]
    fn an_animation_round_trips_through_its_file_form() {
        let p = classic();
        let a = p.get("yellow-waving").unwrap();
        let again = Animation::compile(&a.to_file()).unwrap();
        assert_eq!(&again, a);
    }

    #[test]
    fn bad_animations_are_refused_with_the_reason() {
        let mut f = classic().get("yellow").unwrap().to_file();
        f.frames[0][3] = "YYYYYYYZ".into();
        assert!(Animation::compile(&f).unwrap_err().contains("\"Z\""));
        f.frames[0][3] = "YYYYYYY".into();
        assert!(Animation::compile(&f).unwrap_err().contains("not 8 characters"));
        let mut g = classic().get("yellow").unwrap().to_file();
        g.palette.remove(".");
        assert!(Animation::compile(&g).unwrap_err().contains("off"));
    }

    #[test]
    fn a_looping_animation_wraps_and_a_one_shot_holds_its_last_frame() {
        let mut a = classic().get("yellow-waving").unwrap().clone();
        let n = a.frames.len() as u64;
        let per = 1000 / a.fps as u64;
        assert_eq!(a.frame_at(per * n), &a.frames[0]);
        a.looped = false;
        assert_eq!(a.frame_at(per * n * 5), a.frames.last().unwrap());
    }
}
