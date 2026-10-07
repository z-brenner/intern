//! Peak resident memory while one document is processed - sampled, not
//! measured.
//!
//! On Linux a thread reads `VmRSS` from `/proc/<pid>/status` every 25 ms for
//! the parser worker this process started, the model server, and this
//! process itself, and keeps the highest figure seen for each. A spike
//! shorter than the interval can be missed, which is why the report calls
//! these sampled peaks. Elsewhere nothing is sampled and every peak is
//! `null`: absent, not zero.
//!
//! The worker is recognised as a child of this process named
//! `intern-worker`; the server by the process id passed in, or failing
//! that by the name `llama-server` - on a machine running more than one
//! server, pass the id.

use serde::{Deserialize, Serialize};

/// How often a sampler reads memory.
pub const INTERVAL_MS: u64 = 25;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MemoryPeaks {
    #[serde(default)]
    pub worker_mb: Option<f64>,
    #[serde(default)]
    pub server_mb: Option<f64>,
    #[serde(default)]
    pub bench_mb: Option<f64>,
}

impl MemoryPeaks {
    pub fn is_empty(&self) -> bool {
        self.worker_mb.is_none() && self.server_mb.is_none() && self.bench_mb.is_none()
    }
}

pub use platform::MemorySampler;

#[cfg(target_os = "linux")]
mod platform {
    use std::{
        fs,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread::{self, JoinHandle},
        time::Duration,
    };

    use super::{INTERVAL_MS, MemoryPeaks};

    /// Samples until [`MemorySampler::finish`].
    pub struct MemorySampler {
        stop: Arc<AtomicBool>,
        handle: Option<JoinHandle<MemoryPeaks>>,
    }

    impl MemorySampler {
        pub fn start(server_pid: Option<u32>) -> Self {
            let stop = Arc::new(AtomicBool::new(false));
            let flag = Arc::clone(&stop);
            let handle = thread::Builder::new()
                .name("memory-sampler".into())
                .spawn(move || sample_until(&flag, server_pid))
                .ok();
            Self { stop, handle }
        }

        pub fn finish(mut self) -> MemoryPeaks {
            self.stop.store(true, Ordering::SeqCst);
            self.handle
                .take()
                .and_then(|handle| handle.join().ok())
                .unwrap_or_default()
        }
    }

    impl Drop for MemorySampler {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::SeqCst);
        }
    }

    #[derive(Default)]
    struct Watched {
        workers: Vec<u32>,
        servers: Vec<u32>,
    }

    fn sample_until(stop: &AtomicBool, server_pid: Option<u32>) -> MemoryPeaks {
        let own = std::process::id();
        let mut peaks = MemoryPeaks::default();
        let mut watched = Watched::default();
        let mut tick = 0_u64;
        loop {
            // Listing every process is the expensive part; the worker can
            // restart mid-document, so it is still redone twice a second.
            if tick % (500 / INTERVAL_MS) == 0 {
                watched = discover(own, server_pid);
            }
            sample(&mut peaks, &watched);
            if stop.load(Ordering::SeqCst) {
                // One last reading at the moment the document finished.
                sample(&mut peaks, &watched);
                return peaks;
            }
            thread::sleep(Duration::from_millis(INTERVAL_MS));
            tick += 1;
        }
    }

    fn sample(peaks: &mut MemoryPeaks, watched: &Watched) {
        let total = |pids: &[u32]| {
            let readings = pids
                .iter()
                .filter_map(|pid| resident_kb(&format!("/proc/{pid}/status")))
                .collect::<Vec<_>>();
            (!readings.is_empty()).then(|| readings.iter().sum::<u64>())
        };
        raise(&mut peaks.worker_mb, total(&watched.workers));
        raise(&mut peaks.server_mb, total(&watched.servers));
        raise(&mut peaks.bench_mb, resident_kb("/proc/self/status"));
    }

    fn raise(peak: &mut Option<f64>, kilobytes: Option<u64>) {
        if let Some(kilobytes) = kilobytes {
            let megabytes = (kilobytes as f64 / 1024.0 * 10.0).round() / 10.0;
            *peak = Some(peak.map_or(megabytes, |current| current.max(megabytes)));
        }
    }

    fn discover(own: u32, server_pid: Option<u32>) -> Watched {
        let mut watched = Watched::default();
        let Ok(entries) = fs::read_dir("/proc") else {
            return watched;
        };
        for entry in entries.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            if server_pid == Some(pid) {
                watched.servers.push(pid);
                continue;
            }
            let Ok(comm) = fs::read_to_string(format!("/proc/{pid}/comm")) else {
                continue;
            };
            match comm.trim() {
                "intern-worker" if parent(pid) == Some(own) => watched.workers.push(pid),
                "llama-server" if server_pid.is_none() => watched.servers.push(pid),
                _ => {}
            }
        }
        watched
    }

    fn parent(pid: u32) -> Option<u32> {
        status_field(&format!("/proc/{pid}/status"), "PPid:")
    }

    fn resident_kb(path: &str) -> Option<u64> {
        status_field(path, "VmRSS:").map(u64::from)
    }

    fn status_field(path: &str, field: &str) -> Option<u32> {
        let status = fs::read_to_string(path).ok()?;
        status
            .lines()
            .find_map(|line| line.strip_prefix(field))?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::MemoryPeaks;

    /// No sampling off Linux: every peak is reported as unknown.
    pub struct MemorySampler;

    impl MemorySampler {
        pub fn start(_server_pid: Option<u32>) -> Self {
            Self
        }

        pub fn finish(self) -> MemoryPeaks {
            MemoryPeaks::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn the_sampler_sees_this_process_and_no_worker() {
        let sampler = MemorySampler::start(Some(u32::MAX));
        let ballast = vec![1_u8; 8 << 20];
        std::thread::sleep(std::time::Duration::from_millis(3 * INTERVAL_MS));
        let peaks = sampler.finish();
        assert!(ballast.iter().all(|byte| *byte == 1));
        assert!(
            peaks.bench_mb.is_some_and(|megabytes| megabytes > 8.0),
            "{peaks:?}"
        );
        assert_eq!(peaks.worker_mb, None, "this process started no worker");
        assert_eq!(peaks.server_mb, None, "no process has that id");
    }

    #[test]
    fn an_unsampled_run_has_no_peaks_rather_than_zero_ones() {
        assert!(MemoryPeaks::default().is_empty());
        let text = serde_json::to_string(&MemoryPeaks::default()).unwrap();
        assert_eq!(
            text,
            r#"{"worker_mb":null,"server_mb":null,"bench_mb":null}"#
        );
    }
}
