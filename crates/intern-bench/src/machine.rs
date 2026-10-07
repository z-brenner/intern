//! What a run was measured on: the machine, the model file, the commit.
//!
//! Latency means nothing without the machine it was taken on, and a score
//! means little without the model and the code that earned it, so every
//! report and recording carries them.

use std::{
    io::Read,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MachineInfo {
    #[serde(default)]
    pub cpu: Option<String>,
    #[serde(default)]
    pub logical_cores: Option<usize>,
    #[serde(default)]
    pub total_ram_mb: Option<u64>,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub os_version: Option<String>,
}

impl MachineInfo {
    pub fn current() -> Self {
        Self {
            cpu: cpu_model(),
            logical_cores: std::thread::available_parallelism().ok().map(usize::from),
            total_ram_mb: total_ram_mb(),
            os: std::env::consts::OS.to_owned(),
            os_version: os_version(),
        }
    }

    /// One line for a report header.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(cpu) = &self.cpu {
            parts.push(cpu.clone());
        }
        if let Some(cores) = self.logical_cores {
            parts.push(format!("{cores} logical cores"));
        }
        if let Some(ram) = self.total_ram_mb {
            parts.push(format!("{:.1} GB RAM", ram as f64 / 1024.0));
        }
        let os = match &self.os_version {
            Some(version) => format!("{} ({version})", self.os),
            None => self.os.clone(),
        };
        if !os.is_empty() {
            parts.push(os);
        }
        if parts.is_empty() {
            "unknown".to_owned()
        } else {
            parts.join(", ")
        }
    }
}

fn cpu_model() -> Option<String> {
    if let Ok(identifier) = std::env::var("PROCESSOR_IDENTIFIER") {
        return Some(identifier);
    }
    let info = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    info.lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().to_owned())
}

fn total_ram_mb() -> Option<u64> {
    let info = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = info.lines().find(|line| line.starts_with("MemTotal:"))?;
    let kilobytes = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    Some(kilobytes / 1024)
}

fn os_version() -> Option<String> {
    let release = std::fs::read_to_string("/etc/os-release").ok()?;
    release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches('"').to_owned())
}

/// The model a run asked, and the file behind it when one was named.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ModelInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub sha256: Option<String>,
}

impl ModelInfo {
    /// Hashes the model file in pieces: it is gigabytes, and reading it
    /// whole into memory beside a loaded model is how a laptop swaps.
    pub fn with_file(id: &str, path: Option<&Path>) -> Result<Self, String> {
        let mut info = Self {
            id: id.to_owned(),
            ..Self::default()
        };
        let Some(path) = path else {
            return Ok(info);
        };
        let mut file = std::fs::File::open(path)
            .map_err(|error| format!("cannot open model {}: {error}", path.display()))?;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 1 << 20];
        let mut size = 0_u64;
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| format!("cannot read model {}: {error}", path.display()))?;
            if read == 0 {
                break;
            }
            size += read as u64;
            hasher.update(&buffer[..read]);
        }
        info.path = Some(path.display().to_string());
        info.size_bytes = Some(size);
        info.sha256 = Some(format!("{:x}", hasher.finalize()));
        Ok(info)
    }
}

/// `git rev-parse HEAD` in the working directory, with `-dirty` appended
/// when tracked files have uncommitted changes.
pub fn git_commit() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .is_some_and(|status| status.status.success() && !status.stdout.is_empty());
    Some(if dirty {
        format!("{commit}-dirty")
    } else {
        commit
    })
}

/// Now, as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    utc_from_unix(seconds)
}

/// A Unix time as `YYYY-MM-DDTHH:MM:SSZ`, by the proleptic Gregorian
/// calendar (Howard Hinnant's `civil_from_days`).
pub fn utc_from_unix(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let rest = seconds % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_times_format_as_utc_dates() {
        assert_eq!(utc_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_from_unix(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc_from_unix(1_790_000_000), "2026-09-21T14:13:20Z");
        assert_eq!(utc_from_unix(4_107_542_399), "2100-02-28T23:59:59Z");
    }

    #[test]
    fn a_model_file_is_hashed_and_measured() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("model.gguf");
        std::fs::write(&path, b"abc").unwrap();
        let info = ModelInfo::with_file("intern-local", Some(&path)).unwrap();
        assert_eq!(info.size_bytes, Some(3));
        assert_eq!(
            info.sha256.as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
        let bare = ModelInfo::with_file("intern-local", None).unwrap();
        assert_eq!(bare.sha256, None);
    }

    #[test]
    fn the_machine_summary_reads_as_one_line() {
        let machine = MachineInfo {
            cpu: Some("Example CPU".into()),
            logical_cores: Some(4),
            total_ram_mb: Some(16_384),
            os: "linux".into(),
            os_version: Some("Example OS 1".into()),
        };
        assert_eq!(
            machine.summary(),
            "Example CPU, 4 logical cores, 16.0 GB RAM, linux (Example OS 1)"
        );
        assert_eq!(MachineInfo::default().summary(), "unknown");
    }
}
