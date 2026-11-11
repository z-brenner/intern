//! The memory the machine has available, read as the app's tier plan reads
//! it (`intern_engine::tiers::available_memory_bytes`), so the worker can
//! decide what it may spend beside a loaded model.

/// The memory available now, in bytes, or `None` when it cannot be read.
/// Never panics.
///
/// On Linux, the kernel's own estimate (`MemAvailable`): free memory plus the
/// page cache and reclaimable slabs it could give up without swapping,
/// mapped or not, a loaded model's weights among them. On Windows,
/// `GlobalMemoryStatusEx`'s available physical memory: the free, zero and
/// standby lists, which do not count the pages a process holds in its
/// working set, a loaded model's weights among them.
pub fn available_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .as_deref()
            .and_then(mem_available_from_meminfo)
    }
    #[cfg(windows)]
    {
        windows_memory::available_physical()
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        None
    }
}

/// `MemAvailable` from the text of `/proc/meminfo`, in bytes.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn mem_available_from_meminfo(meminfo: &str) -> Option<u64> {
    let line = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))?;
    let mut words = line.split_whitespace();
    let amount = words.next()?.parse::<u64>().ok()?;
    match words.next() {
        Some("kB") => amount.checked_mul(1024),
        None => Some(amount),
        Some(_) => None,
    }
}

// One Win32 call with no safe wrapper in the worker's dependencies; the
// same call, the same way, as the app's tier plan makes it.
#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_memory {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    pub(super) fn available_physical() -> Option<u64> {
        let mut status = MEMORYSTATUSEX {
            dwLength: u32::try_from(size_of::<MEMORYSTATUSEX>()).ok()?,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailVirtual: 0,
            ullAvailExtendedVirtual: 0,
        };
        // SAFETY: `status` is a correctly sized MEMORYSTATUSEX with its
        // length set, as the call requires, and lives for the whole call.
        // Failure is reported through the return value, which is checked.
        let succeeded = unsafe { GlobalMemoryStatusEx(&mut status) };
        (succeeded != 0).then_some(status.ullAvailPhys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_available_is_read_in_bytes() {
        let meminfo = "MemTotal:       16314748 kB\nMemFree:          812344 kB\nMemAvailable:    9011236 kB\n";
        assert_eq!(mem_available_from_meminfo(meminfo), Some(9_011_236 * 1024));
        assert_eq!(mem_available_from_meminfo("MemAvailable: 42\n"), Some(42));
        assert_eq!(mem_available_from_meminfo("MemAvailable: 42 MB\n"), None);
        assert_eq!(mem_available_from_meminfo("MemFree: 42 kB\n"), None);
    }

    /// Where the machine says, it says something.
    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn the_machine_reports_its_memory() {
        assert!(available_bytes().is_some_and(|bytes| bytes > 0));
    }

    /// The worker reads the memory as the app's tier plan reads it: this is
    /// a copy of `intern_engine::tiers::available_memory_bytes`, which the
    /// worker does not link, so the two are read side by side here and must
    /// agree to within what the machine's other work moves in between.
    #[test]
    fn the_worker_reads_the_memory_as_the_tier_plan_does() {
        const SLACK: u64 = 256 * 1024 * 1024;
        let before = intern_engine::tiers::available_memory_bytes();
        let worker = available_bytes();
        let after = intern_engine::tiers::available_memory_bytes();
        match (before, worker, after) {
            (Some(before), Some(worker), Some(after)) => {
                let low = before.min(after).saturating_sub(SLACK);
                let high = before.max(after).saturating_add(SLACK);
                assert!(
                    (low..=high).contains(&worker),
                    "{worker} against {before} and {after}"
                );
            }
            (None, None, None) => {}
            readings => panic!("one read the memory and the other did not: {readings:?}"),
        }
    }
}
