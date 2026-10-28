//! The release zip: what `pcplugin pack` writes and the checks the app runs before it installs one.

use std::io::{Cursor, Read, Write};
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::manifest::{self, EXTENSIONS, MAX_BYTES, MAX_ENTRIES, Package};

pub struct Built {
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub sha256: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Builds `<id>-<version>.zip`. Same inputs, same bytes: entries are sorted and carry a fixed date,
/// so CI can rebuild a release and compare hashes.
pub fn build(pkg: &Package, wasm: Option<&Path>) -> Result<Built> {
    let m = &pkg.manifest;
    let mut files: Vec<(String, Vec<u8>)> = vec![("plugin.json".into(), std::fs::read(pkg.dir.join("plugin.json"))?)];
    if let Some(entry) = &m.entry {
        let path = wasm.map(Path::to_path_buf).unwrap_or_else(|| pkg.dir.join(entry));
        let bytes = std::fs::read(&path).with_context(|| format!("reading the plugin's wasm at {} (build it, or pass --wasm)", path.display()))?;
        if !bytes.starts_with(b"\0asm") {
            bail!("{} is not a WebAssembly file", path.display());
        }
        files.push((entry.clone(), bytes));
    }
    for rel in &m.packs {
        for f in manifest::walk(&pkg.dir.join(rel)) {
            let name = f.strip_prefix(&pkg.dir)?.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/");
            files.push((name, std::fs::read(&f)?));
        }
    }
    for (src, dst) in [("README.md", "README.md"), ("CHANGELOG.md", "CHANGELOG.md"), ("LICENSE", "LICENSE.txt"), ("icon.png", "icon.png")] {
        if let Ok(bytes) = std::fs::read(pkg.dir.join(src)) {
            files.push((dst.into(), bytes));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::DEFAULT)
        .unix_permissions(0o644);
    for (name, bytes) in &files {
        zip.start_file(name, options)?;
        zip.write_all(bytes)?;
    }
    let bytes = zip.finish()?.into_inner();
    check(&bytes).context("the package this built would be refused by the app")?;
    Ok(Built { file_name: format!("{}-{}.zip", m.id, m.version), sha256: sha256_hex(&bytes), bytes })
}

/// What the app checks after the sha256 and before it extracts anything (docs/releasing.md).
/// Returns the manifest inside.
pub fn check(zip_bytes: &[u8]) -> Result<Value> {
    let mut zip = ZipArchive::new(Cursor::new(zip_bytes)).context("not a zip file")?;
    if zip.len() > MAX_ENTRIES {
        bail!("{} entries; the limit is {MAX_ENTRIES}", zip.len());
    }
    let mut total = 0u64;
    let mut manifest = None;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i)?;
        let name = f.name()?.into_owned();
        if f.is_symlink() {
            bail!("{name}: symlinks are refused");
        }
        let path = Path::new(&name);
        let unsafe_path = name.contains('\\')
            || name.contains(':')
            || path.components().any(|c| !matches!(c, Component::Normal(_)))
            || f.enclosed_name().is_none();
        if unsafe_path {
            bail!("{name}: absolute paths, \"..\", drive letters and backslashes are refused");
        }
        if f.is_dir() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !EXTENSIONS.contains(&ext) {
            bail!("{name}: .{ext} is not allowed ({})", EXTENSIONS.join(" "));
        }
        // Count what actually comes out, not what the header claims.
        let mut buf = Vec::new();
        (&mut f).take(MAX_BYTES + 1 - total).read_to_end(&mut buf)?;
        total += buf.len() as u64;
        if total > MAX_BYTES {
            bail!("unpacks to more than {} MiB", MAX_BYTES / 1024 / 1024);
        }
        if name == "plugin.json" {
            manifest = Some(serde_json::from_slice::<Value>(&buf).context("plugin.json")?);
        }
    }
    manifest.context("no plugin.json at the root of the zip")
}

/// The release entry for `registry/<id>.json`. The registry's CI recomputes sha256 and bytes from
/// the GitHub asset; these are what the author should expect it to find.
pub fn registry_release(pkg: &Package, built: &Built, tag: &str) -> Value {
    json!({
        "version": pkg.manifest.version,
        "tag": tag,
        "asset": built.file_name,
        "sha256": built.sha256,
        "bytes": built.bytes.len(),
        "api": pkg.manifest.api.clone().unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn a_plain_package_passes() {
        let z = zip_of(&[("plugin.json", br#"{"id":"a.b"}"#), ("x.wasm", b"\0asm"), ("content/p/pack.json", b"{}")]);
        assert_eq!(check(&z).unwrap()["id"], "a.b");
    }

    #[test]
    fn escapes_and_odd_files_are_refused() {
        for bad in ["../evil.json", "/abs.json", "C:/x.json", "a\\b.json", "run.exe", "script.js"] {
            let z = zip_of(&[("plugin.json", b"{}"), (bad, b"x")]);
            assert!(check(&z).is_err(), "{bad} was accepted");
        }
    }

    #[test]
    fn a_zip_without_a_manifest_is_refused() {
        assert!(check(&zip_of(&[("README.md", b"hi")])).unwrap_err().to_string().contains("plugin.json"));
    }

    #[test]
    fn too_many_entries_are_refused() {
        let names: Vec<String> = (0..=MAX_ENTRIES).map(|i| format!("f{i}.txt")).collect();
        let entries: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), &b""[..])).collect();
        assert!(check(&zip_of(&entries)).unwrap_err().to_string().contains("entries"));
    }

    #[test]
    fn the_same_package_builds_to_the_same_bytes() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/flags");
        let pkg = manifest::load(&dir).unwrap();
        let tmp = std::env::temp_dir().join("pcplugin-test.wasm");
        std::fs::write(&tmp, b"\0asm\x01\0\0\0").unwrap();
        let a = build(&pkg, Some(&tmp)).unwrap();
        let b = build(&pkg, Some(&tmp)).unwrap();
        assert_eq!(a.sha256, b.sha256);
        assert_eq!(a.file_name, "pitcrew.flags-1.0.0.zip");
        let inside = check(&a.bytes).unwrap();
        assert_eq!(inside["id"], "pitcrew.flags");
    }
}
