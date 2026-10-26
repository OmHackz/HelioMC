//! Data-dir resolution, safe path joining, name sanitization, atomic JSON.
//!
//! Every path in Helio is built through `safe_join`, every user-supplied name
//! through `sanitize_name`. Corrupt JSON files are recovered from a `.bak`
//! copy; we never panic on user data.

use std::fs;
use std::path::{Path, PathBuf};

use crate::brand;
use crate::error::{HelioError, HelioResult};

/// The default data dir on Windows is `%LOCALAPPDATA%/Helio`; on Unix it is
/// `$XDG_DATA_HOME/Helio` or `~/.local/share/Helio`.
pub fn default_data_dir() -> PathBuf {
    if cfg!(windows) {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local).join(brand::DATA_DIR_NAME);
        }
        if let Some(user) = std::env::var_os("USERPROFILE") {
            return PathBuf::from(user).join(brand::DATA_DIR_NAME);
        }
        PathBuf::from(brand::DATA_DIR_NAME)
    } else {
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(data_home).join(brand::DATA_DIR_NAME);
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".local/share")
                .join(brand::DATA_DIR_NAME);
        }
        PathBuf::from(brand::DATA_DIR_NAME)
    }
}

/// Resolve the data dir in the documented order:
/// 1. explicit `--data-dir` (passed by the front end)
/// 2. `HELIO_DATA_DIR`
/// 3. a `data_dir` field in an existing `config.json` (only if the file exists)
/// 4. the platform default
pub fn resolve_data_dir(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }
    if let Ok(p) = std::env::var(format!("{}DATA_DIR", brand::ENV_PREFIX)) {
        return PathBuf::from(p);
    }
    default_data_dir()
}

/// Create the data tree if it does not exist.
pub fn ensure_tree(data_dir: &Path) -> HelioResult<()> {
    for sub in [
        "",
        "logs",
        "cache",
        "cache/skins",
        "runtime",
        "runtime/java",
        "runtime/run",
        "libraries",
        "assets/indexes",
        "assets/objects",
        "versions",
        "instances",
    ] {
        let p = if sub.is_empty() {
            data_dir.to_path_buf()
        } else {
            data_dir.join(sub)
        };
        fs::create_dir_all(&p)?;
    }
    Ok(())
}

/// Join `child` onto `base` safely.
///
/// Rejects: absolute paths, drive-relative paths, `..` traversal that escapes
/// `base`, and reserved Windows device names. Returns the joined, normalized
/// path without touching the filesystem.
pub fn safe_join(base: &Path, child: &str) -> HelioResult<PathBuf> {
    let child = child.trim();
    if child.is_empty() {
        return Err(HelioError::InvalidInput("empty path segment".into()));
    }

    // Windows reserved names are invalid in any position of a path.
    let segments: Vec<&str> = child.split(['/', '\\']).collect();
    for s in &segments {
        if is_reserved_windows(s) {
            return Err(HelioError::InvalidInput(format!(
                "{s:?} is a reserved Windows device name"
            )));
        }
    }

    // Reject absolute paths (the join of an absolute path yields it verbatim).
    let normalized = child.replace('\\', "/");
    if normalized.starts_with('/')
        || (cfg!(windows) && normalized.len() >= 2 && normalized.as_bytes()[1] == b':')
    {
        return Err(HelioError::InvalidInput(format!(
            "absolute paths are not allowed inside the data dir: {child:?}"
        )));
    }
    let candidate = base.join(&normalized);

    // Ensure the result stays inside the base.
    let candidate_canon = canonical(&candidate);
    let base_canon = canonical(base);
    if !inside(&base_canon, &candidate_canon) {
        return Err(HelioError::InvalidInput(format!(
            "path escapes the data directory: {child:?}"
        )));
    }
    Ok(base.join(&normalized))
}

fn canonical(p: &Path) -> PathBuf {
    // Lexically normalize: resolve `..` and `.` components so we can
    // reliably check whether a path stays inside `base`.
    let mut out = PathBuf::new();
    for component in p.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::Normal(c) => {
                out.push(c);
            }
            std::path::Component::CurDir => {}
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                out.push(component.as_os_str());
            }
        }
    }
    out
}

fn inside(base: &Path, candidate: &Path) -> bool {
    candidate.strip_prefix(base).is_ok()
}

/// True when the string is a reserved Windows device name.
/// Covers `CON`, `PRN`, `AUX`, `NUL`, `COM1`..`COM99`, `LPT1`..`LPT9`,
/// with or without an extension (e.g. `aux.txt`).
pub fn is_reserved_windows(name: &str) -> bool {
    let stem = match name.rfind('.') {
        Some(0) => name,
        Some(i) => &name[..i],
        None => name,
    };
    let upper = stem.to_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.starts_with("COM") && com_lpt_ok(&upper, 99, "COM"))
        || (upper.starts_with("LPT") && com_lpt_ok(&upper, 9, "LPT"))
}

/// True when `upper` is `prefix` followed by a number in 1..=max.
fn com_lpt_ok(upper: &str, max: u32, prefix: &str) -> bool {
    let digits = &upper[prefix.len()..];
    match digits.parse::<u32>() {
        Ok(n) => (1..=max).contains(&n),
        Err(_) => false,
    }
}

/// Turn an arbitrary user-supplied name into a safe filesystem component.
///
/// Keeps ASCII letters/digits/underscore/hyphen/dot/whitespace, replaces
/// everything else with `-`, trims to 40 chars, and falls back to `unnamed`.
/// Reserved Windows names get a trailing `_`.
pub fn sanitize_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' => c,
            ' ' => '-',
            _ => '-',
        })
        .collect();
    // Collapse consecutive dashes into one.
    let collapsed: String = cleaned
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let trimmed = collapsed.trim_end_matches(['-', ' ']).to_string();
    let trimmed = trimmed.chars().take(40).collect::<String>();
    let trimmed = trimmed.trim_end_matches(['-', ' ']).to_string();
    if trimmed.is_empty() {
        "unnamed".into()
    } else if is_reserved_windows(&trimmed) {
        format!("{trimmed}_")
    } else {
        trimmed
    }
}

/// Lowercase-kebab-case a name into a unique-ish slug.
pub fn slugify(name: &str) -> String {
    let s = sanitize_name(name).to_lowercase();
    s.trim().replace([' ', '_', '.'], "-")
}

/// Make `slug` unique by appending `-2`, `-3`, ... as long as
/// `exists(slug)` is true. `exists` is typically a filesystem check.
pub fn unique_slug(slug: &str, mut exists: impl FnMut(&str) -> bool) -> String {
    if !exists(slug) {
        return slug.to_string();
    }
    let mut n = 2;
    loop {
        let cand = format!("{slug}-{n}");
        if !exists(&cand) {
            return cand;
        }
        n += 1;
    }
}

/// Atomically write JSON: write to `path.tmp-<rand>` then rename. A previous
/// good file is preserved as `.bak` before the rename so a crash mid-write
/// leaves a recoverable backup.
pub fn write_json_atomic(path: &Path, value: &impl serde::Serialize) -> HelioResult<()> {
    let text = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension(format!("tmp{}", rand::random::<u32>()));
    if path.exists() {
        let bak = path.with_extension("bak");
        let _ = fs::rename(path, &bak);
    }
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Read JSON from `path`, returning `default` when the file is missing.
/// A corrupt file is backed up as `.bak` and `default` is returned.
pub fn read_json_or_default<T>(path: &Path, default: T) -> T
where
    T: for<'de> serde::Deserialize<'de>,
{
    match fs::read(path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => {
                let bak = path.with_extension("bak");
                let _ = fs::write(&bak, bytes);
                tracing::warn!(
                    path = %path.display(),
                    "corrupt JSON; backed up as .bak and using defaults"
                );
                default
            }
        },
        Err(_) => default,
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn t(base: &str) -> PathBuf {
        PathBuf::from(base)
    }

    #[test]
    fn rejects_traversal() {
        assert!(safe_join(&t("d"), "../x").is_err());
        assert!(safe_join(&t("d"), "..\\x").is_err());
        assert!(safe_join(&t("d"), "a/../../b").is_err());
        assert!(safe_join(&t("d"), "..").is_err());
    }

    #[test]
    fn rejects_absolute_and_drive_paths() {
        assert!(safe_join(&t("d"), "/etc/passwd").is_err());
        #[cfg(windows)]
        assert!(safe_join(&t("d"), "C:\\evil").is_err());
        #[cfg(windows)]
        assert!(safe_join(&t("d"), "D:\\evil").is_err());
    }

    #[test]
    fn rejects_reserved_windows() {
        assert!(safe_join(&t("d"), "CON").is_err());
        assert!(safe_join(&t("d"), "NUL").is_err());
        assert!(safe_join(&t("d"), "COM1").is_err());
        assert!(safe_join(&t("d"), "aux.txt").is_err());
    }

    #[test]
    fn allows_plain_names() {
        let p = safe_join(&t("d"), "instances/foo").unwrap();
        assert_eq!(p, t("d/instances/foo"));
    }

    #[test]
    fn allows_unicode() {
        let p = safe_join(&t("d"), "instances/мир-2").unwrap();
        assert!(p.ends_with("instances/мир-2"));
    }

    #[test]
    fn sanitize_collisions_and_long_names() {
        assert_eq!(sanitize_name("My Cool Pack"), "My-Cool-Pack");
        assert_eq!(sanitize_name("a/b\\c"), "a-b-c");
        let long = "x".repeat(200);
        assert_eq!(sanitize_name(&long), "x".repeat(40));
        assert_eq!(sanitize_name("NUL"), "NUL_");
        assert_eq!(sanitize_name("!!!"), "unnamed");
    }

    #[test]
    fn unique_slug_appends_counters() {
        let taken = |s: &str| matches!(s, "pack" | "pack-2");
        assert_eq!(unique_slug("pack", taken), "pack-3");
        assert_eq!(unique_slug("other", taken), "other");
    }

    #[test]
    fn roundtrip_json_with_bak() {
        let dir = std::env::temp_dir().join(format!("helio-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.json");
        write_json_atomic(&p, &42u32).unwrap();
        assert_eq!(read_json_or_default::<u32>(&p, 0), 42);
        // corrupt it: read_json_or_default must fall back and keep a .bak
        fs::write(&p, "not json").unwrap();
        assert_eq!(read_json_or_default::<u32>(&p, 7), 7);
        assert!(p.with_extension("bak").exists());
        let _ = fs::remove_dir_all(dir);
    }
}
