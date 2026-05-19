use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub repo: String,
    pub name: String,
    pub version: String,
    pub installed: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub licenses: Vec<String>,
    pub depends: Vec<String>,
    pub optional_deps: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub repo: String,
    pub installed: bool,
    pub install_date: Option<String>,
    pub installed_size: Option<String>,
    pub build_date: Option<String>,
    pub packager: Option<String>,
}

/// Parse a single line of `yay -Sl` output.
/// Format: `<repo> <name> <version> [installed]`
pub fn parse_yay_sl_line(line: &str) -> Option<Package> {
    let mut parts = line.split_whitespace();
    let repo = parts.next()?.to_string();
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    let rest: Vec<&str> = parts.collect();
    let installed = rest.iter().any(|&s| s == "[installed]" || s == "[installed:");
    Some(Package {
        repo,
        name,
        version,
        installed,
        description: None,
    })
}

/// Parse `yay -Si` / `pacman -Si` output into a PackageInfo.
pub fn parse_yay_si_output(output: &str) -> PackageInfo {
    let mut info = PackageInfo::default();
    for line in output.lines() {
        if let Some((key, val)) = split_si_line(line) {
            match key {
                "Name" => info.name = val.to_string(),
                "Version" => info.version = val.to_string(),
                "Description" => info.description = val.to_string(),
                "URL" => info.url = val.to_string(),
                "Licenses" => info.licenses = split_list(val),
                "Depends On" => info.depends = split_list(val),
                "Optional Deps" => info.optional_deps = split_list(val),
                "Conflicts With" => info.conflicts = split_list(val),
                "Provides" => info.provides = split_list(val),
                "Repository" => info.repo = val.to_string(),
                "Install Date" => info.install_date = Some(val.to_string()),
                "Installed Size" => info.installed_size = Some(val.to_string()),
                "Build Date" => info.build_date = Some(val.to_string()),
                "Packager" => info.packager = Some(val.to_string()),
                _ => {}
            }
        }
    }
    // Determine installed from "Install Date" presence
    info.installed = info.install_date.is_some();
    info
}

fn split_si_line(line: &str) -> Option<(&str, &str)> {
    let idx = line.find(':')?;
    let key = line[..idx].trim();
    let val = line[idx + 1..].trim();
    if key.is_empty() || val.is_empty() {
        return None;
    }
    Some((key, val))
}

fn split_list(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter(|&x| x != "None")
        .map(|x| x.trim_end_matches(':').to_string())
        .collect()
}
