use sevenz_rust2::encoder_options::{EncoderOptions, LZMA2Options};
use sevenz_rust2::{ArchiveEntry, ArchiveWriter, EncoderConfiguration, EncoderMethod, SourceReader};
use std::env;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

pub const ET_VERSION: &str = "v2.6.4";
pub const ET_DESC: &str = "windows-x86_64";

fn et_url() -> String {
    format!("https://github.com/EasyTier/EasyTier/releases/download/{ET_VERSION}/easytier-windows-x86_64-{ET_VERSION}.zip")
}

const KEEP: &[&str] = &[
    "easytier-core.exe",
    "easytier-cli.exe",
    "wintun.dll",
    "Packet.dll",
    "WinDivert64.sys",
];
const ENTRY: &str = "easytier-core.exe";
const CLI: &str = "easytier-cli.exe";

pub fn prepare() {
    println!("cargo::rerun-if-changed=.easytier");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let manifest = PathBuf::from(get_var("CARGO_MANIFEST_DIR").unwrap());
    let base = manifest
        .join(".easytier")
        .join(ET_VERSION)
        .join(ET_DESC);
    let entry_conf = base.join("entry-conf.v1.txt");
    let cli_conf = base.join("cli-conf.v1.txt");
    let archive = base.join("easytier.7z");

    println!("cargo::rustc-env=MCN_ET_ENTRY_CONF={}", entry_conf.display());
    println!("cargo::rustc-env=MCN_ET_CLI_CONF={}", cli_conf.display());
    println!("cargo::rustc-env=MCN_ET_ARCHIVE={}", archive.display());

    if entry_conf.is_file() && cli_conf.is_file() && archive.is_file() {
        return;
    }

    if base.exists() {
        fs::remove_dir_all(&base).expect("clear stale EasyTier cache dir");
    }
    fs::create_dir_all(&base).expect("create EasyTier cache dir");

    let zip_path = env::temp_dir().join(format!("mcn-build-{}.zip", process::id()));
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .expect("build EasyTier download client");
    let mut resp = client
        .get(et_url())
        .send()
        .and_then(|r| r.error_for_status())
        .expect("download EasyTier release zip");
    resp.copy_to(&mut fs::File::create(&zip_path).expect("create temp zip file"))
        .inspect_err(|_| {
            let _ = fs::remove_file(&zip_path);
        })
        .expect("write EasyTier release zip to temp file");

    repack(&zip_path, &archive);
    let _ = fs::remove_file(&zip_path);
    fs::write(&entry_conf, ENTRY).expect("write entry conf");
    fs::write(&cli_conf, CLI).expect("write cli conf");
}

fn repack(zip_path: &Path, archive: &Path) {
    let mut zip = zip::ZipArchive::new(fs::File::open(zip_path).unwrap()).unwrap();
    let tmp = archive.with_extension("7z.tmp");
    let mut writer = ArchiveWriter::new(fs::File::create(&tmp).unwrap()).unwrap();
    writer.set_content_methods(vec![
        EncoderConfiguration {
            method: EncoderMethod::LZMA2,
            options: Some(EncoderOptions::LZMA2(LZMA2Options::from_level(9))),
        },
        EncoderConfiguration {
            method: EncoderMethod::BCJ_X86_FILTER,
            options: None,
        },
    ]);

    let mut entries: Vec<ArchiveEntry> = vec![];
    let mut readers: Vec<SourceReader<Cursor<Vec<u8>>>> = vec![];
    for keep in KEEP {
        let full = format!("easytier-{ET_DESC}/{keep}");
        let mut entry = zip
            .by_name(&full)
            .unwrap_or_else(|e| panic!("EasyTier zip missing `{full}`: {e}"));
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).unwrap();
        entries.push(ArchiveEntry::new_file(*keep));
        readers.push(SourceReader::new(Cursor::new(buf)));
    }
    writer.push_archive_entries(entries, readers).unwrap();
    writer.finish().unwrap();
    fs::rename(&tmp, archive).unwrap();
}

fn get_var<K: AsRef<std::ffi::OsStr>>(key: K) -> Result<String, env::VarError> {
    println!("cargo::rerun-if-env-changed={}", key.as_ref().to_string_lossy());
    env::var(key.as_ref())
}
