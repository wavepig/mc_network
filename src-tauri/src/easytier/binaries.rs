use std::path::{Path, PathBuf};

pub struct EasyTierPaths {
    pub core: PathBuf,
    pub cli: PathBuf,
    pub dir: PathBuf,
}

static ARCHIVE: (&str, &str, &[u8]) = (
    include_str!(env!("MCN_ET_ENTRY_CONF")),
    include_str!(env!("MCN_ET_CLI_CONF")),
    include_bytes!(env!("MCN_ET_ARCHIVE")),
);

use std::sync::OnceLock;

static PATHS: OnceLock<EasyTierPaths> = OnceLock::new();

fn extract_into(dir: &Path) {
    std::fs::create_dir_all(dir).expect("extract embedded EasyTier binaries: create dir");
    let reader = std::io::Cursor::new(ARCHIVE.2);
    sevenz_rust2::decompress(reader, dir).expect("extract embedded EasyTier binaries: decompress");
}

pub fn acquire() -> &'static EasyTierPaths {
    let paths = PATHS.get_or_init(|| {
        let dir = std::env::temp_dir()
            .join("mc-network")
            .join(format!("embedded-easytier-{}", std::process::id()));
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        extract_into(&dir);
        EasyTierPaths {
            core: dir.join(ARCHIVE.0.trim()),
            cli: dir.join(ARCHIVE.1.trim()),
            dir,
        }
    });
    if !paths.core.is_file() {
        extract_into(&paths.dir);
    }
    paths
}

pub fn cleanup() {
    if let Some(p) = PATHS.get() {
        let _ = std::fs::remove_dir_all(&p.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_extracts_core_and_cli() {
        let paths = acquire();
        assert!(paths.core.is_file(), "core missing: {:?}", paths.core);
        assert!(paths.cli.is_file(), "cli missing: {:?}", paths.cli);
        assert_eq!(
            paths.dir,
            std::env::temp_dir()
                .join("mc-network")
                .join(format!("embedded-easytier-{}", std::process::id()))
        );
    }

    #[test]
    fn acquire_is_idempotent() {
        let a = acquire();
        let b = acquire();
        assert!(std::ptr::eq(a, b));
        assert_eq!(a.core, b.core);
    }
}
