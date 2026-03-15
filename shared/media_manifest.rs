use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StillAssetEntry {
    pub key: String,
    pub label: String,
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    pub path: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionAssetEntry {
    pub key: String,
    pub label: String,
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    pub frame_delay_ms: u16,
    pub symbol: String,
    pub frames: Vec<PathBuf>,
    pub manifest: BTreeMap<String, String>,
}

pub fn collect_stills(stills_root: &Path, manifests_root: &Path) -> Vec<StillAssetEntry> {
    let mut paths = read_sorted_paths(stills_root, |path| {
        path.extension().and_then(|v| v.to_str()) == Some("rgb565")
    });
    let mut entries = Vec::new();

    for path in paths.drain(..) {
        let key = file_stem_string(&path);
        let manifest = load_manifest(&manifests_root.join(format!("{key}.txt")));
        let (width, height) =
            parse_size(manifest.get("firmware_size").map(String::as_str), 120, 90);
        let scale = parse_u16(manifest.get("firmware_scale").map(String::as_str), 2);

        entries.push(StillAssetEntry {
            key: key.clone(),
            label: title_label(&key),
            width,
            height,
            scale,
            path,
        });
    }

    entries
}

pub fn collect_motion_clips(motion_root: &Path, manifests_root: &Path) -> Vec<MotionAssetEntry> {
    let mut entries = Vec::new();
    let mut dirs = read_sorted_paths(motion_root, Path::is_dir);

    for dir in dirs.drain(..) {
        let key = dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("clip")
            .to_string();
        let manifest = load_manifest(&manifests_root.join(format!("{key}.txt")));
        let (width, height) = parse_size(manifest.get("firmware_size").map(String::as_str), 80, 60);
        let scale = parse_u16(manifest.get("firmware_scale").map(String::as_str), 3);
        let delay_cs = parse_u16(manifest.get("frame_delay_cs").map(String::as_str), 10);
        let frames = read_sorted_paths(&dir, |path| {
            path.extension().and_then(|v| v.to_str()) == Some("rgb565")
        });

        entries.push(MotionAssetEntry {
            key: key.clone(),
            label: title_label(&key),
            width,
            height,
            scale,
            frame_delay_ms: delay_cs.saturating_mul(10),
            symbol: frame_symbol(&key),
            frames,
            manifest,
        });
    }

    entries
}

pub fn read_sorted_paths<F>(root: &Path, predicate: F) -> Vec<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let mut paths = Vec::new();
    if let Ok(read_dir) = fs::read_dir(root) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if predicate(&path) {
                paths.push(path);
            }
        }
        paths.sort();
    }
    paths
}

pub fn load_manifest(path: &Path) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return values,
    };

    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        values.insert(key.trim().to_string(), value.trim().to_string());
    }

    values
}

pub fn parse_size(value: Option<&str>, default_w: u16, default_h: u16) -> (u16, u16) {
    let Some(value) = value else {
        return (default_w, default_h);
    };
    let Some((width, height)) = value.split_once('x') else {
        return (default_w, default_h);
    };
    (
        width.parse::<u16>().unwrap_or(default_w),
        height.parse::<u16>().unwrap_or(default_h),
    )
}

pub fn parse_u16(value: Option<&str>, default_value: u16) -> u16 {
    value
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(default_value)
}

pub fn file_stem_string(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("asset")
        .to_string()
}

pub fn title_label(key: &str) -> String {
    key.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut title = String::new();
                    title.extend(first.to_uppercase());
                    title.push_str(chars.as_str());
                    title
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn frame_symbol(key: &str) -> String {
    let mut symbol = String::from("MOTION_");
    for ch in key.chars() {
        if ch.is_ascii_alphanumeric() {
            symbol.push(ch.to_ascii_uppercase());
        } else {
            symbol.push('_');
        }
    }
    symbol.push_str("_FRAMES");
    symbol
}
