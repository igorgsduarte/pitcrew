//! `pcplugin run`: a small PitCrew host on your machine. The imports, permission checks and limits
//! match the app's; a scripted flag stands in for the sim, and a fake PitFlag device stands in for
//! the matrix unless the port setting names a real one.

use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Component as PathPart, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Map, Value};
use wasmtime::component::{Component, HasSelf, Linker, Resource, ResourceTable};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder, Trap, UpdateDeadline};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::device::{self, FakeDevice};
use crate::manifest::{Manifest, Package};

wasmtime::component::bindgen!({
    world: "plugin",
    path: "../../wit",
    with: { "pitcrew:plugin/serial.port": PortHandle },
    imports: { default: trappable },
});

use pitcrew::plugin::{content, host, serial, settings, types as t, ui};

pub const CALL_BUDGET_MS: u64 = 5;
pub const INIT_BUDGET_MS: u64 = 250;
pub const MEMORY_BYTES: usize = 16 << 20;
pub const UI_EVERY_MS: u64 = 200;
pub const MAX_OWN_FILE: usize = 1 << 20;
pub const FAKE_PORT: &str = "fake";

pub enum PortHandle {
    Fake,
    Real(Box<dyn serialport::SerialPort>),
}

pub struct Options {
    pub wasm: Option<PathBuf>,
    pub port: String,
    pub script: String,
    pub seconds: Option<f64>,
    pub plain: bool,
    pub ui: bool,
    pub sets: Vec<(String, Value)>,
    pub events: Vec<Scheduled>,
    pub data: Option<PathBuf>,
}

/// Something the driver does at a given time: an `actions` button, or a click on the plugin's screen.
#[derive(Debug, Clone)]
pub enum Scheduled {
    Action { at_ms: u64, id: String },
    Press { at_ms: u64, node: String },
    Select { at_ms: u64, node: String, value: String },
}

impl Scheduled {
    fn at(&self) -> u64 {
        match self {
            Scheduled::Action { at_ms, .. } | Scheduled::Press { at_ms, .. } | Scheduled::Select { at_ms, .. } => *at_ms,
        }
    }
}

struct Pack {
    id: String,
    kind: String,
    dir: PathBuf,
}

struct State {
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
    dir: PathBuf,
    manifest: Manifest,
    settings: Map<String, Value>,
    device: FakeDevice,
    data: PathBuf,
    ui_dirty: bool,
    /// The call that runs now: when it started, its budget, and the time spent in host imports.
    /// Host time (a real serial port can take 15 ms to open) does not count against the budget.
    call_started: Instant,
    call_budget: Duration,
    host_time: Duration,
}

impl State {
    fn guest_time(&self) -> Duration {
        self.call_started.elapsed().saturating_sub(self.host_time)
    }

    fn start_call(&mut self, budget_ms: u64) {
        self.call_started = Instant::now();
        self.call_budget = Duration::from_millis(budget_ms);
        self.host_time = Duration::ZERO;
    }

    fn in_host<R>(&mut self, f: impl FnOnce(&mut State) -> R) -> R {
        let t = Instant::now();
        let r = f(self);
        self.host_time += t.elapsed();
        r
    }
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.wasi, table: &mut self.table }
    }
}

impl State {
    fn has(&self, permission: &str) -> bool {
        self.manifest.permissions.iter().any(|p| p == permission)
    }

    fn packs(&self) -> Vec<Pack> {
        let bundled = self.manifest.packs.iter().map(|rel| self.dir.join(rel));
        let own = std::fs::read_dir(self.data.join("packs")).into_iter().flatten().flatten().map(|e| e.path());
        bundled
            .chain(own)
            .filter_map(|dir| {
                let v: Value = serde_json::from_slice(&std::fs::read(dir.join("pack.json")).ok()?).ok()?;
                Some(Pack { id: v["id"].as_str()?.to_string(), kind: v["type"].as_str()?.to_string(), dir })
            })
            .collect()
    }
}

fn status_name(level: t::StatusLevel) -> &'static str {
    match level {
        t::StatusLevel::Ok => "ok",
        t::StatusLevel::Idle => "idle",
        t::StatusLevel::Warning => "warning",
        t::StatusLevel::Error => "error",
    }
}

/// A relative path that stays inside its folder, or None.
pub fn safe_rel(path: &str) -> Option<PathBuf> {
    let p = Path::new(path);
    let ok = !path.is_empty() && !path.contains(['\\', ':']) && p.components().all(|c| matches!(c, PathPart::Normal(_)));
    ok.then(|| p.to_path_buf())
}

impl t::Host for State {}
impl ui::Host for State {}

impl host::Host for State {
    fn log(&mut self, level: t::LogLevel, text: String) -> wasmtime::Result<()> {
        let level = match level {
            t::LogLevel::Debug => "debug",
            t::LogLevel::Info => "info",
            t::LogLevel::Warn => "warn",
            t::LogLevel::Error => "error",
        };
        println!("        {level:<5} {text}");
        Ok(())
    }

    fn set_status(&mut self, level: t::StatusLevel, text: String) -> wasmtime::Result<()> {
        println!("        status {}: {text}", status_name(level));
        Ok(())
    }

    fn invalidate_ui(&mut self) -> wasmtime::Result<()> {
        self.ui_dirty = true;
        Ok(())
    }
}

impl serial::Host for State {
    fn open(&mut self, setting_key: String, baud: u32) -> wasmtime::Result<Result<Resource<PortHandle>, String>> {
        if !self.has("devices:serial") {
            return Ok(Err("This plugin does not have the devices:serial permission".into()));
        }
        if !self.manifest.settings.iter().any(|s| s.key == setting_key && s.kind == "serialPort") {
            return Ok(Err(format!("\"{setting_key}\" is not a serialPort setting in plugin.json")));
        }
        let chosen = self.settings.get(&setting_key).and_then(Value::as_str).map(str::to_string);
        let handle = match chosen.as_deref() {
            None => return Ok(Err("No port chosen. Pick one in the plugin's settings".into())),
            Some(FAKE_PORT) => {
                self.device = FakeDevice::boot();
                PortHandle::Fake
            }
            Some(name) => match self.in_host(|_| serialport::new(name, baud).timeout(Duration::from_millis(5)).open()) {
                Ok(p) => PortHandle::Real(p),
                Err(e) => return Ok(Err(format!("Could not open {name}: {e}"))),
            },
        };
        Ok(Ok(self.table.push(handle)?))
    }
}

impl serial::HostPort for State {
    fn write(&mut self, port: Resource<PortHandle>, bytes: Vec<u8>) -> wasmtime::Result<Result<(), String>> {
        self.in_host(|s| {
            Ok(match s.table.get_mut(&port)? {
                PortHandle::Fake => {
                    s.device.receive(&bytes);
                    Ok(())
                }
                PortHandle::Real(p) => p.write_all(&bytes).map_err(|e| e.to_string()),
            })
        })
    }

    fn read(&mut self, port: Resource<PortHandle>, max: u32) -> wasmtime::Result<Result<Vec<u8>, String>> {
        self.in_host(|s| {
            Ok(match s.table.get_mut(&port)? {
                PortHandle::Fake => Ok(s.device.take(max as usize)),
                PortHandle::Real(p) => {
                    let waiting = p.bytes_to_read().map_err(|e| e.to_string());
                    waiting.and_then(|n| {
                        let mut buf = vec![0; (n as usize).min(max as usize)];
                        p.read_exact(&mut buf).map(|_| buf).map_err(|e| e.to_string())
                    })
                }
            })
        })
    }

    fn drop(&mut self, port: Resource<PortHandle>) -> wasmtime::Result<()> {
        self.in_host(|s| s.table.delete(port).map(|_| ()))?;
        Ok(())
    }
}

impl settings::Host for State {
    fn get(&mut self) -> wasmtime::Result<String> {
        Ok(Value::Object(self.settings.clone()).to_string())
    }
}

impl content::Host for State {
    fn packs(&mut self, of_type: String) -> wasmtime::Result<Vec<String>> {
        Ok(State::packs(self).into_iter().filter(|p| p.kind == of_type).map(|p| p.id).collect())
    }

    fn read(&mut self, pack: String, path: String) -> wasmtime::Result<Result<Vec<u8>, String>> {
        let Some(p) = State::packs(self).into_iter().find(|p| p.id == pack) else {
            return Ok(Err(format!("No pack \"{pack}\"")));
        };
        let Some(rel) = safe_rel(&path) else {
            return Ok(Err(format!("\"{path}\" is not a relative path inside the pack")));
        };
        Ok(std::fs::read(p.dir.join(rel)).map_err(|e| format!("{pack}/{path}: {e}")))
    }

    fn write_own(&mut self, path: String, bytes: Vec<u8>) -> wasmtime::Result<Result<(), String>> {
        if !self.has("content:write-own") {
            return Ok(Err("This plugin does not have the content:write-own permission".into()));
        }
        let Some(rel) = safe_rel(&path) else {
            return Ok(Err(format!("\"{path}\" is not a relative path inside the plugin's data folder")));
        };
        if bytes.len() > MAX_OWN_FILE {
            return Ok(Err(format!("{path}: more than {} KiB", MAX_OWN_FILE / 1024)));
        }
        let full = self.data.join(rel);
        let written = full.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&full, &bytes));
        println!("        wrote {} ({} bytes)", full.display(), bytes.len());
        Ok(written.map_err(|e| e.to_string()))
    }
}

/// One stretch of the scripted session.
#[derive(Debug, Clone)]
pub struct Step {
    pub label: String,
    pub connected: bool,
    pub signal: t::FlagSignal,
    pub ms: Option<u64>,
}

pub const DEMO: &str = "none:2,ready:1,set:1,go:1,green:4,yellow:3,yellow+waving:3,yellow+debris:3,blue:3,white:3,green+driver-repair:3,red:3,checkered:4";

/// `token[+token…]:seconds, …`. Tokens: `none` (no sim), a flag (`yellow`, `blue`, …), `ready`/`set`/`go`,
/// `waving`, `debris`, `one-lap`, `driver-black`/`driver-repair`/`driver-disqualify`. No seconds: forever.
pub fn parse_script(s: &str) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    for part in s.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let (tokens, secs) = part.split_once(':').unwrap_or((part, ""));
        let ms = if secs.is_empty() {
            None
        } else {
            Some((secs.parse::<f64>().with_context(|| format!("\"{part}\": seconds"))? * 1000.0) as u64)
        };
        let mut sig = t::FlagSignal { primary: None, waving: false, debris: false, one_lap_to_green: false, start: None, driver: vec![] };
        let mut connected = true;
        for tok in tokens.split('+') {
            match tok {
                "none" => connected = false,
                "waving" => sig.waving = true,
                "debris" => sig.debris = true,
                "one-lap" => sig.one_lap_to_green = true,
                "ready" => sig.start = Some(t::StartLight::Ready),
                "set" => sig.start = Some(t::StartLight::Set),
                "go" => {
                    sig.start = Some(t::StartLight::Go);
                    sig.primary = Some(t::Flag::Green);
                }
                "driver-black" => sig.driver.push(t::DriverFlag::Black),
                "driver-repair" => sig.driver.push(t::DriverFlag::Repair),
                "driver-disqualify" => sig.driver.push(t::DriverFlag::Disqualify),
                flag => {
                    sig.primary = Some(match flag {
                        "disqualify" => t::Flag::Disqualify,
                        "black" => t::Flag::Black,
                        "repair" => t::Flag::Repair,
                        "red" => t::Flag::Red,
                        "checkered" => t::Flag::Checkered,
                        "white" => t::Flag::White,
                        "yellow" => t::Flag::Yellow,
                        "blue" => t::Flag::Blue,
                        "green" => t::Flag::Green,
                        other => bail!("\"{other}\" is not a script token (see pcplugin run --help)"),
                    })
                }
            }
        }
        steps.push(Step { label: tokens.to_string(), connected, signal: sig, ms });
    }
    if steps.is_empty() {
        bail!("the script is empty");
    }
    Ok(steps)
}

/// The step at `now_ms`; the script loops.
fn step_at(steps: &[Step], now_ms: u64) -> usize {
    let total: u64 = steps.iter().map(|s| s.ms.unwrap_or(u64::MAX / 2)).fold(0, u64::saturating_add);
    let mut t = now_ms % total.max(1);
    for (i, s) in steps.iter().enumerate() {
        let d = s.ms.unwrap_or(u64::MAX / 2);
        if t < d {
            return i;
        }
        t -= d;
    }
    steps.len() - 1
}

/// The checks the app runs on every tree `render` returns. See docs/ui.md.
pub fn check_tree(nodes: &[ui::UiNode], m: &Manifest) -> Result<(), String> {
    use ui::NodeKind as K;
    if nodes.len() > 500 {
        return Err(format!("{} nodes; the limit is 500", nodes.len()));
    }
    let roots: Vec<_> = nodes.iter().filter(|n| n.parent.is_none()).collect();
    if roots.len() != 1 || !matches!(roots[0].kind, K::Page(_)) {
        return Err("exactly one node without a parent, and it must be a page".into());
    }
    let mut seen = HashSet::new();
    for n in nodes {
        let id = &n.id;
        if id.is_empty() || id.len() > 64 {
            return Err(format!("\"{id}\": ids are 1 to 64 characters"));
        }
        if !seen.insert(id.as_str()) {
            return Err(format!("duplicate id \"{id}\""));
        }
        let parent = match &n.parent {
            Some(p) if !seen.contains(p.as_str()) => return Err(format!("\"{id}\" names parent \"{p}\" before it exists")),
            Some(p) => nodes.iter().find(|x| &x.id == p),
            None => None,
        };
        match &n.kind {
            K::Grid(g) if !(1..=3).contains(&g.columns) => return Err(format!("\"{id}\": grid columns are 1 to 3")),
            K::Table(tb) if tb.rows.len() > 200 => return Err(format!("\"{id}\": more than 200 rows")),
            K::Table(tb) if tb.rows.iter().any(|r| r.len() != tb.columns.len()) => return Err(format!("\"{id}\": a row has a different number of cells than columns")),
            K::Progress(p) if !(0.0..=1.0).contains(&p.value) => return Err(format!("\"{id}\": progress is 0.0 to 1.0")),
            K::PixelGrid(g) => {
                if !(1..=32).contains(&g.width) || !(1..=32).contains(&g.height) {
                    return Err(format!("\"{id}\": a pixel grid is 1 to 32 a side"));
                }
                if g.palette.len() > 16 || g.pixels.len() != g.width as usize * g.height as usize {
                    return Err(format!("\"{id}\": palette ≤ 16 colours and pixels = width × height"));
                }
                if g.pixels.iter().any(|&p| p as usize >= g.palette.len().max(1)) {
                    return Err(format!("\"{id}\": a pixel names a colour the palette does not have"));
                }
            }
            K::Tab(_) if !parent.is_some_and(|p| matches!(p.kind, K::Tabs(_))) => return Err(format!("\"{id}\": a tab must be a child of tabs")),
            K::Tabs(tb) if !nodes.iter().any(|x| x.parent.as_deref() == Some(id) && x.id == tb.selected) => {
                return Err(format!("\"{id}\": selected \"{}\" is not one of its tabs", tb.selected));
            }
            K::SettingsForm(keys) => {
                if let Some(k) = keys.iter().find(|k| !m.settings.iter().any(|s| &&s.key == k)) {
                    return Err(format!("\"{id}\": settings-form names \"{k}\", which is not in plugin.json"));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The tree as an indented outline, for --ui.
pub fn outline(nodes: &[ui::UiNode]) -> String {
    use ui::NodeKind as K;
    fn depth(nodes: &[ui::UiNode], n: &ui::UiNode) -> usize {
        let mut d = 0;
        let mut cur = n.parent.as_deref();
        while let Some(p) = cur {
            d += 1;
            cur = nodes.iter().find(|x| x.id == p).and_then(|x| x.parent.as_deref());
        }
        d
    }
    let mut out = String::new();
    // Children follow their parent in a valid tree, so a pre-order walk keeps the reading order.
    fn walk(nodes: &[ui::UiNode], parent: Option<&str>, out: &mut String) {
        for n in nodes.iter().filter(|n| n.parent.as_deref() == parent) {
            let what = match &n.kind {
                K::Page(p) => format!("page \"{}\"", p.title),
                K::Tabs(t) => format!("tabs (selected: {})", t.selected),
                K::Tab(l) => format!("tab \"{l}\""),
                K::Card(c) => format!("card \"{}\"", c.title.clone().unwrap_or_default()),
                K::Grid(g) => format!("grid {} columns", g.columns),
                K::Stack(_) => "stack".into(),
                K::Separator => "separator".into(),
                K::Text(t) => format!("text \"{}\"", t.text),
                K::Badge(b) => format!("badge {} \"{}\"", status_name(b.level), b.text),
                K::Alert(a) => format!("alert {} \"{}\"", status_name(a.level), a.text),
                K::Table(t) => format!("table {} columns × {} rows", t.columns.len(), t.rows.len()),
                K::Progress(p) => format!("progress {:.0}% \"{}\"", p.value * 100.0, p.label.clone().unwrap_or_default()),
                K::Button(b) => format!("button \"{}\"", b.label),
                K::Switch(s) => format!("switch \"{}\" {}", s.label, s.checked),
                K::Slider(s) => format!("slider \"{}\" = {}", s.label, s.value),
                K::Select(s) => format!("select \"{}\" = {}", s.label, s.value),
                K::Input(i) => format!("input \"{}\"", i.label),
                K::PixelGrid(g) => format!("pixel-grid {}×{}{}", g.width, g.height, if g.editable { " editable" } else { "" }),
                K::SettingsForm(k) => format!("settings-form [{}]", k.join(", ")),
            };
            out.push_str(&format!("{}{} #{}\n", "  ".repeat(depth(nodes, n)), what, n.id));
            walk(nodes, Some(&n.id), out);
        }
    }
    walk(nodes, None, &mut out);
    out
}

#[derive(Default)]
struct Timing {
    calls: u64,
    slowest: Duration,
    slowest_what: String,
}

fn timed<R>(store: &mut Store<State>, timing: &mut Timing, what: &str, budget_ms: u64, f: impl FnOnce(&mut Store<State>) -> wasmtime::Result<R>) -> Result<R> {
    store.data_mut().start_call(budget_ms);
    store.set_epoch_deadline(1);
    let r = f(store);
    let took = store.data().guest_time();
    timing.calls += 1;
    if budget_ms == CALL_BUDGET_MS && took > timing.slowest {
        timing.slowest = took;
        timing.slowest_what = what.to_string();
    }
    r.map_err(|e| {
        let e = anyhow::Error::from(e);
        if format!("{e:?}").contains("interrupt") {
            anyhow!("{what} ran past its {budget_ms} ms budget; the app would stop this plugin")
        } else {
            e.context(format!("{what} trapped; the app would stop this plugin and restart it"))
        }
    })
}

fn initial_settings(m: &Manifest, opts: &Options) -> Result<Map<String, Value>> {
    let mut s = Map::new();
    for def in &m.settings {
        if let Some(d) = &def.default {
            s.insert(def.key.clone(), d.clone());
        }
        if def.kind == "serialPort" {
            s.insert(def.key.clone(), Value::String(opts.port.clone()));
        }
    }
    for (k, v) in &opts.sets {
        if !m.settings.iter().any(|d| &d.key == k) {
            bail!("--set {k}: not a setting in plugin.json");
        }
        s.insert(k.clone(), v.clone());
    }
    Ok(s)
}

pub fn run(pkg: &Package, opts: &Options) -> Result<()> {
    let m = pkg.manifest.clone();
    let Some(entry) = &m.entry else { bail!("{} is a content package; there is nothing to run", m.id) };
    let wasm = opts.wasm.clone().unwrap_or_else(|| pkg.dir.join(entry));
    let steps = parse_script(&opts.script)?;
    let data = opts.data.clone().unwrap_or_else(|| std::env::temp_dir().join("pcplugin").join(&m.id).join("data"));

    let mut config = Config::new();
    config.epoch_interruption(true);
    let engine = Engine::new(&config)?;
    let component = Component::from_file(&engine, &wasm).map_err(anyhow::Error::from).with_context(|| format!("loading {}", wasm.display()))?;
    let mut linker = Linker::<State>::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    Plugin::add_to_linker::<_, HasSelf<_>>(&mut linker, |s| s)?;

    let state = State {
        // No environment, arguments, folders or network: stderr only, for panic messages.
        wasi: WasiCtxBuilder::new().inherit_stderr().build(),
        table: ResourceTable::new(),
        limits: StoreLimitsBuilder::new().memory_size(MEMORY_BYTES).build(),
        dir: pkg.dir.clone(),
        settings: initial_settings(&m, opts)?,
        manifest: m.clone(),
        device: FakeDevice::boot(),
        data: data.clone(),
        ui_dirty: true,
        call_started: Instant::now(),
        call_budget: Duration::from_millis(INIT_BUDGET_MS),
        host_time: Duration::ZERO,
    };
    let mut store = Store::new(&engine, state);
    store.limiter(|s| &mut s.limits);
    // Checked at every epoch tick: stop the call only when its own time, without host imports, is
    // over the budget.
    store.epoch_deadline_callback(|ctx| {
        let s = ctx.data();
        if s.guest_time() > s.call_budget { Err(Trap::Interrupt.into()) } else { Ok(UpdateDeadline::Continue(1)) }
    });

    let stop = Arc::new(AtomicBool::new(false));
    let ticker = {
        let (engine, stop) = (engine.clone(), stop.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(1));
                engine.increment_epoch();
            }
        })
    };

    println!("{} {} · {} · port {} · data {}", m.id, m.version, wasm.display(), opts.port, data.display());
    let mut timing = Timing::default();
    let result = (|| -> Result<()> {
        let plugin = {
            store.data_mut().start_call(INIT_BUDGET_MS);
            store.set_epoch_deadline(1);
            Plugin::instantiate(&mut store, &component, &linker).map_err(anyhow::Error::from).context("instantiating the component")?
        };
        timed(&mut store, &mut timing, "init", INIT_BUDGET_MS, |s| plugin.call_init(s))?.map_err(|e| anyhow!("init refused to start: {e}"))?;

        let tick_every = Duration::from_secs_f64(1.0 / m.tick_hz.unwrap_or(10) as f64);
        let started = Instant::now();
        let mut current = None;
        let mut events = opts.events.clone();
        events.sort_by_key(Scheduled::at);
        let mut events = events.into_iter().peekable();
        let mut last_leds = None;
        let mut last_drawn = 0u64;
        let mut last_render = 0u64;
        let mut tree: Option<Vec<ui::UiNode>> = None;
        for n in 0u32.. {
            let now = started.elapsed().as_millis() as u64;
            if opts.seconds.is_some_and(|s| now as f64 >= s * 1000.0) {
                break;
            }
            let i = step_at(&steps, now);
            if current != Some(i) {
                current = Some(i);
                let step = &steps[i];
                println!("{:>6.1}s  {}", now as f64 / 1000.0, step.label);
                let session = t::SessionInfo { sim: "pcplugin".into(), connected: step.connected, in_car: step.connected, session_kind: Some("race".into()), track: None };
                timed(&mut store, &mut timing, "on-session", CALL_BUDGET_MS, |s| plugin.call_on_session(s, &session))?;
                timed(&mut store, &mut timing, "on-flag", CALL_BUDGET_MS, |s| plugin.call_on_flag(s, &step.signal))?;
            }
            while let Some(e) = events.next_if(|e| e.at() <= now) {
                println!("{:>6.1}s  driver: {e:?}", now as f64 / 1000.0);
                match e {
                    Scheduled::Action { id, .. } => timed(&mut store, &mut timing, "on-action", CALL_BUDGET_MS, |s| plugin.call_on_action(s, &id))?,
                    Scheduled::Press { node, .. } => {
                        let ev = ui::UiEvent { node, kind: ui::EventKind::Pressed };
                        timed(&mut store, &mut timing, "on-ui-event", CALL_BUDGET_MS, |s| plugin.call_on_ui_event(s, &ev))?
                    }
                    Scheduled::Select { node, value, .. } => {
                        let ev = ui::UiEvent { node, kind: ui::EventKind::Selected(value) };
                        timed(&mut store, &mut timing, "on-ui-event", CALL_BUDGET_MS, |s| plugin.call_on_ui_event(s, &ev))?
                    }
                }
            }
            timed(&mut store, &mut timing, "tick", CALL_BUDGET_MS, |s| plugin.call_tick(s, now))?;
            if store.data().ui_dirty && now.saturating_sub(last_render) >= UI_EVERY_MS {
                store.data_mut().ui_dirty = false;
                last_render = now;
                let t = timed(&mut store, &mut timing, "render", CALL_BUDGET_MS, |s| plugin.call_render(s))?;
                check_tree(&t, &m).map_err(|e| anyhow!("render returned a screen the app would refuse: {e}"))?;
                tree = Some(t);
            }
            if opts.port == FAKE_PORT && store.data().has("devices:serial") && now.saturating_sub(last_drawn) >= 100 {
                let leds = store.data().device.leds();
                if last_leds != Some(leds) {
                    last_leds = Some(leds);
                    last_drawn = now;
                    print!("{}", device::draw(&leds, opts.plain));
                    println!();
                }
            }
            let next = started + tick_every * (n + 1);
            if let Some(wait) = next.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
        }
        timed(&mut store, &mut timing, "on-stop", CALL_BUDGET_MS, |s| plugin.call_on_stop(s))?;
        if opts.ui {
            match &tree {
                Some(t) => println!("\nLast screen ({} nodes, accepted):\n{}", t.len(), outline(t)),
                None => println!("\nThe plugin never asked to draw a screen."),
            }
        }
        Ok(())
    })();
    stop.store(true, Ordering::Relaxed);
    let _ = ticker.join();

    let d = &store.data().device;
    println!(
        "\n{} calls, slowest {} at {:.2} ms (budget {CALL_BUDGET_MS} ms). Device: {} packets, {} frames, {} bytes.",
        timing.calls,
        timing.slowest_what,
        timing.slowest.as_secs_f64() * 1000.0,
        d.packets,
        d.frames,
        d.bytes
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_script_parses_and_loops() {
        let s = parse_script(DEMO).unwrap();
        assert!(!s[0].connected);
        assert_eq!(s[3].signal.start, Some(t::StartLight::Go));
        assert_eq!(s[3].signal.primary, Some(t::Flag::Green));
        assert_eq!(step_at(&s, 0), 0);
        assert_eq!(step_at(&s, 2_500), 1);
        let total: u64 = s.iter().map(|x| x.ms.unwrap()).sum();
        assert_eq!(step_at(&s, total + 10), 0);
    }

    #[test]
    fn a_step_without_seconds_lasts_forever() {
        let s = parse_script("yellow+waving").unwrap();
        assert!(s[0].signal.waving);
        assert_eq!(step_at(&s, 10_000_000), 0);
        assert!(parse_script("purple:2").is_err());
    }

    #[test]
    fn paths_stay_inside() {
        assert!(safe_rel("packs/my-pack/pack.json").is_some());
        for bad in ["", "../x", "/etc/passwd", "C:/x", "a\\b", "a/../../b"] {
            assert!(safe_rel(bad).is_none(), "{bad}");
        }
    }
}
