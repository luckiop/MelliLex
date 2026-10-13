use std::env;
use std::fs;
use std::io::{Cursor, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let plugin_dir = build_kokoro_plugin();
    copy_kokoro_files(&plugin_dir);
    tauri_build::build();
    embed_windows_test_manifest();
}

fn build_kokoro_plugin() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = manifest_dir.join("target").join("kokoro-plugin");
    println!("cargo:rerun-if-changed=crates/mellilex_kokoro/src/lib.rs");
    println!("cargo:rerun-if-changed=crates/mellilex_kokoro/Cargo.toml");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.current_dir(&manifest_dir)
        .arg("build")
        .arg("-p")
        .arg("mellilex_kokoro")
        .arg("--target-dir")
        .arg(&target_dir);
    if profile == "release" {
        cmd.arg("--release");
    } else if profile != "debug" {
        cmd.arg("--profile").arg(&profile);
    }
    let scrubbed: Vec<String> = env::vars()
        .map(|(key, _)| key)
        .filter(|key| {
            key != "CARGO_HOME"
                && (key.starts_with("CARGO")
                    || matches!(
                        key.as_str(),
                        "OUT_DIR" | "PROFILE" | "DEBUG" | "OPT_LEVEL" | "TARGET" | "HOST" | "RUSTC" | "RUSTDOC" | "NUM_JOBS"
                    ))
        })
        .collect();
    for key in scrubbed {
        cmd.env_remove(key);
    }

    let status = cmd
        .stdin(std::process::Stdio::null())
        .status()
        .unwrap_or_else(|err| panic!("Failed to start the speech component build: {err}"));
    if !status.success() {
        panic!("Failed to build the Kokoro speech component");
    }

    let profile_name = if profile == "release" { "release" } else { "debug" };
    target_dir.join(profile_name)
}

fn copy_kokoro_files(plugin_dir: &Path) {
    let plugin = plugin_dir.join("mellilex_kokoro.dll");
    if !plugin.is_file() {
        panic!("Kokoro speech component was not produced at {}", plugin.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let profile_dir = out_dir
        .ancestors()
        .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(profile.as_str()))
        .unwrap_or(&out_dir)
        .to_path_buf();
    let mut dest_dirs = vec![profile_dir.clone(), profile_dir.join("deps")];
    if profile == "release" {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
        dest_dirs.push(manifest_dir.join("resources"));
    }
    for dest_dir in dest_dirs {
        copy_file(&plugin, &dest_dir.join("mellilex_kokoro.dll"));
    }

    for name in [
        "sherpa-onnx-c-api.dll",
        "onnxruntime.dll",
        "onnxruntime_providers_shared.dll",
    ] {
        let source = plugin_dir.join(name);
        if source.is_file() {
            copy_file(&source, &profile_dir.join(name));
        }
    }
}

fn copy_file(source: &Path, dest: &Path) {
    if source == dest {
        return;
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| panic!("Failed to create {}: {err}", parent.display()));
    }
    fs::copy(source, dest).unwrap_or_else(|err| {
        panic!("Failed to copy {} to {}: {err}", source.display(), dest.display())
    });
}

const TEST_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>
"#;

/// Test executables are not produced by tauri-build, so they keep rustc's
/// default manifest. That manifest does not activate comctl32 v6, and the
/// loader then fails `TaskDialogIndirect` before any test can start.
fn embed_windows_test_manifest() {
    if env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("windows") {
        return;
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let manifest_path = out_dir.join("test-manifest.xml");
    fs::write(&manifest_path, TEST_MANIFEST).expect("write test manifest");

    match env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default().as_str() {
        "msvc" => {
            println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
            println!(
                "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
                manifest_path.display()
            );
            println!("cargo:rustc-link-arg-tests=/MANIFESTUAC:NO");
        }
        "gnu" | "gnullvm" => {
            let object = coff_manifest(TEST_MANIFEST.as_bytes());
            let archive = out_dir.join("libtest_manifest.a");
            fs::write(&archive, ar_archive(b"test-manifest.o", &object)).expect("write manifest archive");
            // Search path only. The archive is pulled in by a `#[cfg(test)]`
            // link attribute, so the application binary keeps tauri-build's manifest.
            println!("cargo:rustc-link-search=native={}", out_dir.display());
        }
        _ => {}
    }
}

fn ar_archive(name: &[u8], object: &[u8]) -> Vec<u8> {
    let mut header = [b' '; 60];
    let mut name_field = [b' '; 16];
    name_field[..name.len()].copy_from_slice(name);
    name_field[name.len()] = b'/';
    header[..16].copy_from_slice(&name_field);
    header[16..28].copy_from_slice(b"0           ");
    header[28..34].copy_from_slice(b"0     ");
    header[34..40].copy_from_slice(b"0     ");
    header[40..48].copy_from_slice(b"644     ");
    let size = format!("{:<10}", object.len());
    header[48..58].copy_from_slice(size.as_bytes());
    header[58] = b'`';
    header[59] = b'\n';
    let mut archive = b"!<arch>\n".to_vec();
    archive.extend_from_slice(&header);
    archive.extend_from_slice(object);
    if object.len() % 2 == 1 {
        archive.push(b'\n');
    }
    archive
}

fn coff_manifest(manifest: &[u8]) -> Vec<u8> {
    let mut obj = CoffWriter::new(Cursor::new(Vec::with_capacity(4096)));
    obj.add_data(&resource_directory_table(1));
    obj.add_data(&resource_directory_id_entry(24, 24, true));
    obj.add_data(&resource_directory_table(1));
    obj.add_data(&resource_directory_id_entry(1, 48, true));
    obj.add_data(&resource_directory_table(1));
    obj.add_data(&resource_directory_id_entry(1033, 72, false));
    let address = obj.add_data(&resource_data_entry(88, manifest.len() as u32));
    obj.add_data(manifest);
    obj.align_to(8);
    obj.add_relocation(address);
    obj.finish().into_inner()
}

struct CoffWriter<W> {
    writer: W,
    size_of_raw_data: u32,
    number_of_relocations: u16,
}

impl<W: Write + Seek> CoffWriter<W> {
    fn new(mut writer: W) -> Self {
        writer.write_all(&[0; 60]).expect("reserve coff header");
        Self {
            writer,
            size_of_raw_data: 0,
            number_of_relocations: 0,
        }
    }

    fn add_data(&mut self, data: &[u8]) -> u32 {
        let start = self.size_of_raw_data;
        self.writer.write_all(data).expect("write coff data");
        self.size_of_raw_data = start + data.len() as u32;
        start
    }

    fn align_to(&mut self, n: u32) {
        let offset = self.size_of_raw_data % n;
        if offset != 0 {
            let padding = n - offset;
            self.writer
                .write_all(&vec![0; padding as usize])
                .expect("pad coff section");
            self.size_of_raw_data += padding;
        }
    }

    fn add_relocation(&mut self, address: u32) {
        self.number_of_relocations += 1;
        self.writer
            .write_all(&address.to_le_bytes())
            .expect("write relocation");
        self.writer.write_all(&[0, 0, 0, 0]).expect("write symbol index");
        // IMAGE_REL_AMD64_ADDR32NB
        self.writer.write_all(&3u16.to_le_bytes()).expect("write relocation type");
    }

    fn finish(mut self) -> W {
        let pointer_to_symbol_table = self.writer.stream_position().expect("symbol table offset") as u32;
        self.writer.write_all(b".rsrc\0\0\0").expect("symbol name");
        self.writer.write_all(&[0, 0, 0, 0]).expect("symbol value");
        self.writer.write_all(&[1, 0]).expect("section number");
        self.writer.write_all(&[0, 0, 3, 1]).expect("symbol type");
        self.writer
            .write_all(&self.size_of_raw_data.to_le_bytes())
            .expect("aux size");
        self.writer
            .write_all(&self.number_of_relocations.to_le_bytes())
            .expect("aux relocations");
        self.writer.write_all(&[0; 12]).expect("aux remainder");
        self.writer.write_all(&[4, 0, 0, 0]).expect("string table size");

        self.writer.seek(SeekFrom::Start(0)).expect("rewind");
        self.writer.write_all(&0x8664u16.to_le_bytes()).expect("machine");
        self.writer.write_all(&[1, 0]).expect("section count");
        self.writer.write_all(&[0, 0, 0, 0]).expect("timestamp");
        self.writer
            .write_all(&pointer_to_symbol_table.to_le_bytes())
            .expect("symbol table pointer");
        self.writer.write_all(&[2, 0, 0, 0]).expect("symbol count");
        self.writer.write_all(&[0; 4]).expect("optional header");
        self.writer.write_all(b".rsrc\0\0\0").expect("section name");
        self.writer.write_all(&[0; 8]).expect("virtual size");
        self.writer
            .write_all(&self.size_of_raw_data.to_le_bytes())
            .expect("raw size");
        self.writer.write_all(&60u32.to_le_bytes()).expect("raw pointer");
        self.writer
            .write_all(&(self.size_of_raw_data + 60).to_le_bytes())
            .expect("relocation pointer");
        self.writer.write_all(&[0; 4]).expect("line number pointer");
        self.writer
            .write_all(&self.number_of_relocations.to_le_bytes())
            .expect("relocation count");
        self.writer.write_all(&[0; 2]).expect("line number count");
        self.writer.write_all(&[0x40, 0, 0x30, 0xc0]).expect("section flags");
        self.writer
    }
}

fn resource_directory_table(number_of_id_entries: u16) -> [u8; 16] {
    let mut table = [0; 16];
    table[14..16].copy_from_slice(&number_of_id_entries.to_le_bytes());
    table
}

fn resource_directory_id_entry(id: u32, offset: u32, subdirectory: bool) -> [u8; 8] {
    let mut entry = [0; 8];
    entry[0..4].copy_from_slice(&id.to_le_bytes());
    let flag: u32 = if subdirectory { 0x8000_0000 } else { 0 };
    entry[4..8].copy_from_slice(&((offset & 0x7fff_ffff) | flag).to_le_bytes());
    entry
}

fn resource_data_entry(rva: u32, size: u32) -> [u8; 16] {
    let mut entry = [0; 16];
    entry[0..4].copy_from_slice(&rva.to_le_bytes());
    entry[4..8].copy_from_slice(&size.to_le_bytes());
    entry
}
