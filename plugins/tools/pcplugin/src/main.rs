//! pcplugin: validate, package and run PitCrew plugins. See docs/getting-started.md.

mod device;
mod manifest;
mod package;
mod run;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use serde_json::Value;

const HELP: &str = "\
pcplugin: validate, package and run PitCrew plugins

USAGE
  pcplugin validate <plugin-dir>
  pcplugin pack     <plugin-dir> [--wasm <file>] [--out <dir>] [--tag <tag>]
  pcplugin check    <zip>
  pcplugin run      <plugin-dir> [options]

validate  Checks plugin.json and the bundled packs, the same way the app does before installing.
pack      Writes <id>-<version>.zip and prints its sha256, size and the release entry for registry/<id>.json.
check     Runs the app's install checks on a release zip (paths, extensions, entry count, unpacked size).
run       Loads the plugin in a local host with the app's limits, feeds it a scripted session and
          shows the matrix of a fake PitFlag device.

RUN OPTIONS
  --wasm <file>          The built component (default: <plugin-dir>/<entry>).
  --port <name>          A real serial port such as COM4, or `fake` (default).
  --script <steps>       `token[+token]:seconds,...` (default: a demo of every flag). Tokens: none,
                         ready, set, go, green, yellow, blue, white, red, checkered, black, repair,
                         disqualify, waving, debris, one-lap, driver-black, driver-repair, driver-disqualify.
  --flag <tokens>        One step that never ends, e.g. --flag yellow+waving.
  --seconds <n>          Stop after n seconds (default: run until Ctrl+C).
  --set <key>=<json>     Override a setting, e.g. --set brightness=80.
  --action <id>@<s>      Press a manifest action at s seconds.
  --press <node>@<s>     Click a button on the plugin's screen.
  --select <node>=<v>@<s>  Pick a tab, table row or option.
  --ui                   Print the plugin's last screen as an outline.
  --plain                Draw the matrix with . and # instead of colour.
  --data <dir>           The plugin's own data folder (default: a temp folder).
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(args: &[String]) -> Result<()> {
    let Some((cmd, rest)) = args.split_first() else {
        print!("{HELP}");
        return Ok(());
    };
    let mut a = Args::new(rest);
    match cmd.as_str() {
        "validate" => {
            let pkg = manifest::load(&a.path()?)?;
            a.done()?;
            let m = &pkg.manifest;
            println!("ok: {} {} ({}, {} setting(s), {} pack(s))", m.id, m.version, m.kind, m.settings.len(), m.packs.len());
        }
        "pack" => {
            let pkg = manifest::load(&a.path()?)?;
            let wasm = a.opt("--wasm").map(PathBuf::from);
            let out = a.opt("--out").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("dist"));
            let tag = a.opt("--tag").unwrap_or_else(|| format!("v{}", pkg.manifest.version));
            a.done()?;
            let built = package::build(&pkg, wasm.as_deref())?;
            std::fs::create_dir_all(&out)?;
            let path = out.join(&built.file_name);
            std::fs::write(&path, &built.bytes)?;
            println!("wrote {} ({} bytes)\nsha256 {}", path.display(), built.bytes.len(), built.sha256);
            println!("release entry for registry/{}.json:\n{}", pkg.manifest.id, serde_json::to_string_pretty(&package::registry_release(&pkg, &built, &tag))?);
        }
        "check" => {
            let path = a.path()?;
            a.done()?;
            let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            let m = package::check(&bytes)?;
            println!("ok: {} {} · sha256 {} · {} bytes", m["id"].as_str().unwrap_or("?"), m["version"].as_str().unwrap_or("?"), package::sha256_hex(&bytes), bytes.len());
        }
        "run" => {
            let pkg = manifest::load(&a.path()?)?;
            let mut events = Vec::new();
            while let Some(v) = a.opt("--action") {
                let (id, at) = at_seconds(&v)?;
                events.push(run::Scheduled::Action { at_ms: at, id });
            }
            while let Some(v) = a.opt("--press") {
                let (node, at) = at_seconds(&v)?;
                events.push(run::Scheduled::Press { at_ms: at, node });
            }
            while let Some(v) = a.opt("--select") {
                let (pair, at) = at_seconds(&v)?;
                let (node, value) = pair.split_once('=').context("--select <node>=<value>@<seconds>")?;
                events.push(run::Scheduled::Select { at_ms: at, node: node.into(), value: value.into() });
            }
            let mut sets = Vec::new();
            while let Some(v) = a.opt("--set") {
                let (k, raw) = v.split_once('=').context("--set <key>=<json>")?;
                let value = serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.into()));
                sets.push((k.to_string(), value));
            }
            let script = match (a.opt("--flag"), a.opt("--script")) {
                (Some(_), Some(_)) => bail!("use --flag or --script, not both"),
                (Some(f), None) => f,
                (None, Some(s)) => s,
                (None, None) => run::DEMO.into(),
            };
            let opts = run::Options {
                wasm: a.opt("--wasm").map(PathBuf::from),
                port: a.opt("--port").unwrap_or_else(|| run::FAKE_PORT.into()),
                script,
                seconds: a.opt("--seconds").map(|s| s.parse()).transpose().context("--seconds")?,
                plain: a.flag("--plain"),
                ui: a.flag("--ui"),
                sets,
                events,
                data: a.opt("--data").map(PathBuf::from),
            };
            a.done()?;
            run::run(&pkg, &opts)?;
        }
        "help" | "--help" | "-h" => print!("{HELP}"),
        other => bail!("unknown command \"{other}\"\n\n{HELP}"),
    }
    Ok(())
}

fn at_seconds(v: &str) -> Result<(String, u64)> {
    let (what, at) = v.rsplit_once('@').with_context(|| format!("\"{v}\": expected <what>@<seconds>"))?;
    let s: f64 = at.parse().with_context(|| format!("\"{v}\": seconds"))?;
    Ok((what.to_string(), (s * 1000.0) as u64))
}

/// Just enough argument parsing: one positional path, then `--name value` and `--flag` options.
struct Args {
    rest: Vec<String>,
}

impl Args {
    fn new(args: &[String]) -> Args {
        Args { rest: args.to_vec() }
    }

    fn path(&mut self) -> Result<PathBuf> {
        let mut i = 0;
        while i < self.rest.len() {
            match self.rest[i].as_str() {
                "--plain" | "--ui" => i += 1,
                a if a.starts_with("--") => i += 2,
                _ => return Ok(PathBuf::from(self.rest.remove(i))),
            }
        }
        bail!("missing <path>")
    }

    fn opt(&mut self, name: &str) -> Option<String> {
        let i = self.rest.iter().position(|a| a == name)?;
        if i + 1 >= self.rest.len() {
            return None;
        }
        self.rest.remove(i);
        Some(self.rest.remove(i))
    }

    fn flag(&mut self, name: &str) -> bool {
        let found = self.rest.iter().position(|a| a == name);
        if let Some(i) = found {
            self.rest.remove(i);
        }
        found.is_some()
    }

    fn done(&self) -> Result<()> {
        match self.rest.first() {
            Some(a) => bail!("unexpected argument \"{a}\" (pcplugin help)"),
            None => Ok(()),
        }
    }
}
