//! plugin.json: the rules the app applies before it installs a package. See docs/manifest.md.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;

pub const API: &str = "pitcrew:plugin@2";
pub const PERMISSIONS: [&str; 5] = ["topic:flag", "topic:session", "devices:serial", "devices:firmware", "content:write-own"];
pub const SETTING_TYPES: [&str; 8] = ["serialPort", "integer", "pin", "number", "boolean", "string", "select", "contentRef"];
pub const EXTENSIONS: [&str; 5] = ["wasm", "json", "md", "png", "txt"];
pub const MAX_ENTRIES: usize = 500;
pub const MAX_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Manifest {
    #[serde(rename = "$schema", default)]
    pub _schema: Option<String>,
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    pub kind: String,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub tick_hz: Option<u32>,
    #[serde(default)]
    pub settings: Vec<Setting>,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub nav: Option<Nav>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub packs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    pub key: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub label: String,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub of: Option<String>,
    /// One line under the label.
    #[serde(default)]
    pub description: Option<String>,
    /// Shown after a number, such as "mA" or "s".
    #[serde(default)]
    pub unit: Option<String>,
    /// The increment of a slider or number field.
    #[serde(default)]
    pub step: Option<f64>,
    /// How the window draws the setting: "slider", "percent" or "segmented".
    #[serde(default)]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nav {
    pub label: String,
    pub icon: String,
}

/// A loaded, valid package folder.
pub struct Package {
    pub dir: PathBuf,
    pub manifest: Manifest,
}

pub fn dotted_id(s: &str) -> bool {
    let parts: Vec<_> = s.split('.').collect();
    s.len() <= 64
        && parts.len() >= 2
        && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
}

fn number(p: &str) -> bool {
    !p.is_empty() && p.len() <= 6 && p.chars().all(|c| c.is_ascii_digit())
}

/// `MAJOR.MINOR.PATCH` (stable channel) or `MAJOR.MINOR.PATCH-beta.N` (beta channel).
pub fn semver(s: &str) -> bool {
    let (core, pre) = match s.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (s, None),
    };
    let parts: Vec<_> = core.split('.').collect();
    let core_ok = parts.len() == 3 && parts.iter().all(|p| number(p));
    core_ok && pre.is_none_or(|p| p.strip_prefix("beta.").is_some_and(number))
}

/// The release channel a valid version belongs to.
pub fn channel(version: &str) -> &'static str {
    if version.contains("-beta.") { "beta" } else { "stable" }
}

fn setting_key(s: &str) -> bool {
    let mut c = s.chars();
    s.len() <= 32 && c.next().is_some_and(|f| f.is_ascii_lowercase()) && c.all(|c| c.is_ascii_alphanumeric())
}

pub fn load(dir: &Path) -> Result<Package> {
    let path = dir.join("plugin.json");
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let manifest: Manifest = serde_json::from_str(&text).with_context(|| format!("{} does not match the manifest format", path.display()))?;
    let problems = check(&manifest, dir);
    if !problems.is_empty() {
        bail!("plugin.json has {} problem(s):\n  - {}", problems.len(), problems.join("\n  - "));
    }
    Ok(Package { dir: dir.to_path_buf(), manifest })
}

/// Every problem, not just the first, so an author fixes them in one pass.
pub fn check(m: &Manifest, dir: &Path) -> Vec<String> {
    let mut p = Vec::new();
    if !dotted_id(&m.id) {
        p.push(format!("id \"{}\": lowercase letters, digits and dashes, in at least two dot-separated parts (\"you.my-plugin\")", m.id));
    }
    if m.name.trim().is_empty() || m.name.chars().count() > 40 {
        p.push("name: 1 to 40 characters".into());
    }
    if !semver(&m.version) {
        p.push(format!("version \"{}\": MAJOR.MINOR.PATCH, or MAJOR.MINOR.PATCH-beta.N for the beta channel", m.version));
    }
    if m.publisher.trim().is_empty() {
        p.push("publisher: required".into());
    }
    if m.description.chars().count() > 300 {
        p.push("description: at most 300 characters".into());
    }
    if m.license.as_deref().is_some_and(|l| l.trim().is_empty()) {
        p.push("license: an SPDX id such as \"MIT\", or leave it out".into());
    }
    if let Some(r) = &m.repository {
        let path = r.strip_prefix("https://github.com/").unwrap_or("");
        if path.split('/').filter(|s| !s.is_empty()).count() != 2 {
            p.push(format!("repository \"{r}\": https://github.com/<owner>/<repo>, where the releases are"));
        }
    }
    match m.kind.as_str() {
        "plugin" => {
            if m.api.as_deref() != Some(API) {
                p.push(format!("api: must be \"{API}\""));
            }
            match &m.entry {
                Some(e) if e.ends_with(".wasm") && !e.contains(['/', '\\']) => {}
                _ => p.push("entry: a file name ending in .wasm, at the package root".into()),
            }
            if m.extends.is_some() {
                p.push("extends: only for \"kind\": \"content\"".into());
            }
            if let Some(hz) = m.tick_hz {
                if !(1..=60).contains(&hz) {
                    p.push("tickHz: 1 to 60".into());
                }
            }
        }
        "content" => {
            if m.entry.is_some() || m.api.is_some() || !m.permissions.is_empty() || m.tick_hz.is_some() || m.nav.is_some() {
                p.push("a content package has no entry, api, permissions, tickHz or nav: nothing in it runs".into());
            }
            if !m.extends.as_deref().is_some_and(dotted_id) {
                p.push("extends: the content type this package adds to, e.g. \"flags.animations\"".into());
            }
            if m.packs.is_empty() {
                p.push("packs: a content package lists at least one pack folder".into());
            }
        }
        other => p.push(format!("kind \"{other}\": \"plugin\" or \"content\"")),
    }
    let mut seen = HashSet::new();
    for perm in &m.permissions {
        if !PERMISSIONS.contains(&perm.as_str()) {
            p.push(format!("permissions: unknown \"{perm}\" (known: {})", PERMISSIONS.join(", ")));
        }
        if !seen.insert(perm) {
            p.push(format!("permissions: \"{perm}\" twice"));
        }
    }
    let mut keys = HashSet::new();
    for s in &m.settings {
        let k = &s.key;
        if !setting_key(k) {
            p.push(format!("settings \"{k}\": key is camelCase, starts with a lowercase letter, at most 32 characters"));
        }
        if !keys.insert(k.as_str()) {
            p.push(format!("settings \"{k}\": key used twice"));
        }
        if s.label.trim().is_empty() || s.label.chars().count() > 40 {
            p.push(format!("settings \"{k}\": label is 1 to 40 characters"));
        }
        if !SETTING_TYPES.contains(&s.kind.as_str()) {
            p.push(format!("settings \"{k}\": unknown type \"{}\" (known: {})", s.kind, SETTING_TYPES.join(", ")));
        }
        match s.kind.as_str() {
            "serialPort" if !m.permissions.iter().any(|x| x == "devices:serial") => {
                p.push(format!("settings \"{k}\": a serialPort setting needs the devices:serial permission"));
            }
            "integer" | "number" | "pin" => {
                let (lo, hi) = (s.min.unwrap_or(f64::MIN), s.max.unwrap_or(f64::MAX));
                if lo > hi {
                    p.push(format!("settings \"{k}\": min is above max"));
                }
                match s.default.as_ref().map(Value::as_f64) {
                    Some(Some(d)) if d < lo || d > hi => p.push(format!("settings \"{k}\": default is outside min..max")),
                    Some(Some(d)) if s.kind != "number" && d.fract() != 0.0 => p.push(format!("settings \"{k}\": default is not an integer")),
                    Some(None) => p.push(format!("settings \"{k}\": default is not a number")),
                    _ => {}
                }
            }
            "select" => {
                if s.options.is_empty() {
                    p.push(format!("settings \"{k}\": a select needs options"));
                }
                if s.options.iter().any(|o| o.value.is_empty() || o.label.trim().is_empty()) {
                    p.push(format!("settings \"{k}\": every option needs a value and a label"));
                }
                if let Some(d) = &s.default {
                    if !s.options.iter().any(|o| Some(o.value.as_str()) == d.as_str()) {
                        p.push(format!("settings \"{k}\": default is not one of the options"));
                    }
                }
            }
            "contentRef" if !s.of.as_deref().is_some_and(dotted_id) => {
                p.push(format!("settings \"{k}\": a contentRef needs \"of\": the content type"));
            }
            _ => {}
        }
        if s.kind == "pin" && (s.min.is_some_and(|x| x < 0.0) || s.max.is_some_and(|x| x > 63.0)) {
            p.push(format!("settings \"{k}\": a pin is 0 to 63"));
        }
        p.extend(presentation_problems(s));
    }
    let mut ids = HashSet::new();
    for a in &m.actions {
        if a.id.is_empty() || !ids.insert(a.id.as_str()) {
            p.push(format!("actions: id \"{}\" is empty or used twice", a.id));
        }
        if a.label.trim().is_empty() || a.label.chars().count() > 24 {
            p.push(format!("actions \"{}\": label is 1 to 24 characters", a.id));
        }
    }
    if let Some(n) = &m.nav {
        if n.label.trim().is_empty() || n.label.chars().count() > 16 {
            p.push("nav.label: 1 to 16 characters (it is a sidebar item)".into());
        }
        if n.icon.is_empty() || !n.icon.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            p.push("nav.icon: a lucide icon name such as \"flag\" or \"gauge\"".into());
        }
    }
    for t in &m.provides {
        if !dotted_id(t) {
            p.push(format!("provides \"{t}\": a dotted content type name such as \"flags.animations\""));
        }
    }
    let content_types: Vec<&str> = m.provides.iter().map(String::as_str).chain(m.extends.as_deref()).collect();
    for rel in &m.packs {
        p.extend(check_pack(dir, rel, &content_types));
    }
    p
}

/// The ways a window may draw a setting.
pub const DISPLAYS: [&str; 3] = ["slider", "percent", "segmented"];

/// Problems with the fields that only change how a setting is drawn.
pub fn presentation_problems(s: &Setting) -> Vec<String> {
    let k = &s.key;
    let numeric = matches!(s.kind.as_str(), "integer" | "number" | "pin");
    let mut p = Vec::new();
    if let Some(d) = &s.description {
        if d.trim().is_empty() || d.chars().count() > 160 {
            p.push(format!("settings \"{k}\": description is 1 to 160 characters"));
        }
    }
    if let Some(u) = &s.unit {
        if !numeric || u.trim().is_empty() || u.chars().count() > 8 {
            p.push(format!("settings \"{k}\": unit is 1 to 8 characters, on a number setting"));
        }
    }
    if let Some(step) = s.step {
        let whole = s.kind != "number";
        if !numeric || !step.is_finite() || step <= 0.0 || (whole && step.fract() != 0.0) {
            p.push(format!("settings \"{k}\": step is above zero, whole for an integer, on a number setting"));
        }
    }
    match s.display.as_deref() {
        None => {}
        Some("slider" | "percent") if numeric && s.kind != "pin" && s.min.is_some() && s.max.is_some() => {}
        Some("segmented") if s.kind == "select" && (2..=4).contains(&s.options.len()) => {}
        Some("slider" | "percent") => p.push(format!("settings \"{k}\": a slider needs an integer or number with min and max")),
        Some("segmented") => p.push(format!("settings \"{k}\": segmented needs a select with 2 to 4 options")),
        Some(d) => p.push(format!("settings \"{k}\": unknown display \"{d}\" (known: {})", DISPLAYS.join(", "))),
    }
    p
}

fn check_pack(dir: &Path, rel: &str, types: &[&str]) -> Vec<String> {
    let mut p = Vec::new();
    if rel.contains("..") || Path::new(rel).is_absolute() {
        return vec![format!("packs \"{rel}\": a relative folder inside the package")];
    }
    let pack_dir = dir.join(rel);
    let file = pack_dir.join("pack.json");
    let pack: Value = match std::fs::read(&file).map_err(|e| e.to_string()).and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string())) {
        Ok(v) => v,
        Err(e) => return vec![format!("packs \"{rel}\": pack.json: {e}")],
    };
    for field in ["id", "name", "type", "version"] {
        if !pack.get(field).is_some_and(Value::is_string) {
            p.push(format!("packs \"{rel}\": pack.json needs \"{field}\""));
        }
    }
    if let Some(id) = pack.get("id").and_then(Value::as_str) {
        if !dotted_id(id) {
            p.push(format!("packs \"{rel}\": pack id \"{id}\" is not a dotted id"));
        }
    }
    if let Some(t) = pack.get("type").and_then(Value::as_str) {
        if !types.contains(&t) {
            p.push(format!("packs \"{rel}\": type \"{t}\" is not in provides/extends"));
        }
    }
    for f in walk(&pack_dir) {
        let name = f.strip_prefix(dir).unwrap_or(&f).display().to_string();
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !EXTENSIONS.contains(&ext) {
            p.push(format!("{name}: .{ext} files are not allowed in a package ({})", EXTENSIONS.join(" ")));
        } else if ext == "json" {
            if let Err(e) = std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| serde_json::from_slice::<Value>(&b).map_err(|e| e.to_string())) {
                p.push(format!("{name}: {e}"));
            }
        }
    }
    p
}

/// Every file under `dir`, sorted, so packages come out byte-identical.
pub fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/flags")
    }

    fn flags() -> Manifest {
        load(&flags_dir()).expect("the flags manifest is valid").manifest
    }

    #[test]
    fn the_flags_plugin_passes() {
        let m = flags();
        assert_eq!(m.id, "pitcrew.flags");
        assert_eq!(m.nav.unwrap().label, "Flags");
    }

    #[test]
    fn every_problem_is_reported_at_once() {
        let mut m = flags();
        m.id = "Flags".into();
        m.tick_hz = Some(120);
        m.nav = Some(Nav { label: "A very long sidebar label".into(), icon: "Flag!".into() });
        m.permissions.retain(|p| p != "devices:serial");
        let p = check(&m, &flags_dir());
        for needle in ["id \"Flags\"", "tickHz", "nav.label", "nav.icon", "devices:serial"] {
            assert!(p.iter().any(|x| x.contains(needle)), "missing {needle}: {p:?}");
        }
    }

    #[test]
    fn a_content_package_cannot_carry_code() {
        let mut m = flags();
        m.kind = "content".into();
        m.extends = Some("flags.animations".into());
        assert!(check(&m, &flags_dir()).iter().any(|x| x.contains("nothing in it runs")));
    }

    #[test]
    fn ids_and_versions() {
        assert!(dotted_id("pitcrew.flags"));
        assert!(dotted_id("someone.shift-lights"));
        assert!(!dotted_id("flags"));
        assert!(!dotted_id("Some.one"));
        assert!(!dotted_id("a..b"));
        assert!(semver("1.0.0"));
        assert!(semver("1.2.0-beta.3"));
        assert!(!semver("1.0"));
        assert!(!semver("1.0.0-beta"));
        assert!(!semver("1.0.0-beta."));
        assert!(!semver("1.0.0-rc.1"));
        assert!(!semver("1.0.0-beta.1.2"));
        assert_eq!(channel("1.2.0-beta.3"), "beta");
        assert_eq!(channel("1.2.0"), "stable");
    }
}
