//! ONNX Runtime, shared by every engine that runs networks through it
//! (PP-OCR and Indra): the one process-wide binding to the runtime library,
//! the session settings, the errors a failed load or run maps to, and the
//! pool of per-worker session sets.

use std::fmt::Display;
use std::path::Path;
use std::sync::{Condvar, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;

use crate::extract::{CancellationToken, ExtractionError};

/// ONNX Runtime binds once per process, to the library it was first given.
///
/// It is loaded from an absolute path in the runtime directory and never
/// by bare name: a bare name would be resolved through the DLL search
/// order, which looks in places anyone can write to.
static RUNTIME: OnceLock<Result<(), String>> = OnceLock::new();

/// Binds ONNX Runtime from `library`, once per process; later calls report
/// how the first one went, whatever library they name.
pub(crate) fn load_runtime(library: &Path) -> Result<(), ExtractionError> {
    RUNTIME
        .get_or_init(|| {
            let absolute = std::path::absolute(library).map_err(|error| error.to_string())?;
            let builder = ort::init_from(&absolute).map_err(|error| {
                format!(
                    "ONNX Runtime did not load from {}: {error}",
                    absolute.display()
                )
            })?;
            // Telemetry is on by default in the Windows build. Intern is
            // local-first, and a scan's OCR is no one else's business.
            builder
                .with_name("intern-ocr")
                .with_telemetry(false)
                .commit();
            Ok(())
        })
        .clone()
        .map_err(ExtractionError::native_assets_missing)
}

/// A model that will not load is a missing or damaged install: the caller
/// falls back to another engine.
pub(crate) fn model_error(path: &Path, error: impl Display) -> ExtractionError {
    ExtractionError::native_assets_missing(format!(
        "OCR model {} did not load: {error}",
        path.display()
    ))
}

/// A run that fails on networks that loaded - out of memory, almost always -
/// says nothing about the document, so the document may be tried again.
pub(crate) fn run_error(error: impl Display) -> ExtractionError {
    ExtractionError::io(std::io::Error::other(format!(
        "OCR inference failed: {error}"
    )))
}

/// How a session is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SessionSettings {
    /// Threads one run may use.
    pub threads: usize,
    /// Whether the CPU memory arena keeps a run's buffers for the next.
    pub arena: bool,
}

/// A CPU session over the model at `path`: every graph optimization, one
/// inter-op thread, no spinning, no telemetry (see [`load_runtime`]).
pub(crate) fn session(path: &Path, settings: SessionSettings) -> Result<Session, ExtractionError> {
    let error = |error: &dyn Display| model_error(path, error);
    Session::builder()
        .map_err(|e| error(&e))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| error(&e))?
        .with_intra_threads(settings.threads.max(1))
        .map_err(|e| error(&e))?
        .with_inter_threads(1)
        .map_err(|e| error(&e))?
        // A spinning thread waits for work by burning a core. Between the
        // short layers of these networks that is most of the time, on a
        // machine whose user is still working.
        .with_intra_op_spinning(false)
        .map_err(|e| error(&e))?
        .with_inter_op_spinning(false)
        .map_err(|e| error(&e))?
        .with_execution_providers([ort::ep::CPU::default()
            .with_arena_allocator(settings.arena)
            .build()])
        .map_err(|e| error(&e))?
        .commit_from_file(path)
        .map_err(|e| error(&e))
}

struct PoolState<T> {
    idle: Vec<T>,
    created: usize,
}

/// At most `limit` sets of sessions, made as pages need them and kept.
///
/// A page that arrives while every set is busy waits for one, so however
/// many pages a caller reads at once, the threads OCR uses stay within
/// the budget.
pub(crate) struct SessionPool<T> {
    state: Mutex<PoolState<T>>,
    returned: Condvar,
    limit: usize,
}

impl<T> SessionPool<T> {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            state: Mutex::new(PoolState {
                idle: Vec::new(),
                created: 0,
            }),
            returned: Condvar::new(),
            limit: limit.max(1),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PoolState<T>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn acquire(
        &self,
        build: impl Fn() -> Result<T, ExtractionError>,
        cancel: &CancellationToken,
    ) -> Result<Lease<'_, T>, ExtractionError> {
        let mut may_build = true;
        let mut state = self.lock();
        loop {
            if let Some(sessions) = state.idle.pop() {
                return Ok(Lease {
                    pool: self,
                    sessions: Some(sessions),
                });
            }
            if may_build && state.created < self.limit {
                state.created += 1;
                drop(state);
                match build() {
                    Ok(sessions) => {
                        return Ok(Lease {
                            pool: self,
                            sessions: Some(sessions),
                        });
                    }
                    Err(error) => {
                        state = self.lock();
                        state.created -= 1;
                        self.returned.notify_one();
                        // A second set that will not build - memory is
                        // short - is no reason to fail the page while
                        // another set will come back: wait for it.
                        if state.created == 0 {
                            return Err(error);
                        }
                        may_build = false;
                        continue;
                    }
                }
            }
            state = self
                .returned
                .wait_timeout(state, Duration::from_millis(100))
                .unwrap_or_else(PoisonError::into_inner)
                .0;
            cancel.check()?;
            // Every other set failed to build too, or was released: none
            // will come back, so this page builds its own again, and fails
            // with that build's error if it will not.
            if state.created == 0 && state.idle.is_empty() {
                may_build = true;
            }
        }
    }

    fn give_back(&self, sessions: T) {
        self.lock().idle.push(sessions);
        self.returned.notify_one();
    }

    /// Drops every set no page is using, and says how many. A set out on a
    /// lease comes back to the pool as before and goes at the next release;
    /// a page that wants a set after this builds one again.
    pub(crate) fn release_idle(&self) -> usize {
        let mut state = self.lock();
        let idle = std::mem::take(&mut state.idle);
        state.created -= idle.len();
        drop(state);
        self.returned.notify_all();
        // Outside the lock: dropping a session frees its buffers.
        let released = idle.len();
        drop(idle);
        released
    }
}

/// A set of sessions out of the pool, returned when dropped.
pub(crate) struct Lease<'a, T> {
    pool: &'a SessionPool<T>,
    sessions: Option<T>,
}

impl<T> Lease<'_, T> {
    pub(crate) fn sessions(&mut self) -> &mut T {
        self.sessions
            .as_mut()
            .expect("a lease holds its sessions until dropped")
    }
}

impl<T> Drop for Lease<'_, T> {
    fn drop(&mut self) {
        if let Some(sessions) = self.sessions.take() {
            self.pool.give_back(sessions);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A second set of sessions that will not build - memory is short -
    /// makes a page wait for the set another page holds, not fail; only a
    /// pool with no set at all reports the error.
    #[test]
    fn a_set_that_will_not_build_waits_for_one_that_did() {
        let pool = SessionPool::<u32>::new(2);
        let cancel = CancellationToken::new();
        let first = pool.acquire(|| Ok(1), &cancel).unwrap();
        std::thread::scope(|scope| {
            let waiting = scope.spawn(|| {
                let mut lease = pool
                    .acquire(
                        || Err(ExtractionError::resource_limit("no memory")),
                        &cancel,
                    )
                    .expect("waits for the first set instead of failing");
                *lease.sessions()
            });
            std::thread::sleep(Duration::from_millis(250));
            drop(first);
            assert_eq!(waiting.join().unwrap(), 1);
        });

        let empty = SessionPool::<u32>::new(2);
        assert!(
            empty
                .acquire(
                    || Err(ExtractionError::resource_limit("no memory")),
                    &cancel
                )
                .is_err()
        );
    }

    /// Releasing drops the sets no page holds and lets pages build them
    /// again; a set out on a lease is kept until it comes back.
    #[test]
    fn released_sets_are_built_again_when_a_page_wants_one() {
        let pool = SessionPool::<u32>::new(2);
        let cancel = CancellationToken::new();
        let built = std::sync::atomic::AtomicU32::new(0);
        let build = || Ok(built.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        {
            let _first = pool.acquire(build, &cancel).unwrap();
            let _second = pool.acquire(build, &cancel).unwrap();
        }
        assert_eq!(pool.release_idle(), 2);
        assert_eq!(pool.lock().created, 0);

        let held = pool.acquire(build, &cancel).unwrap();
        assert_eq!(built.load(std::sync::atomic::Ordering::SeqCst), 3);
        assert_eq!(pool.release_idle(), 0, "a leased set is not dropped");
        drop(held);
        assert_eq!(pool.release_idle(), 1);
        assert_eq!(pool.lock().created, 0);
    }

    /// Two pages build sets at once into an empty pool and both builds
    /// fail: neither waits for a set that no one is building, and each
    /// page fails with its build's error.
    #[test]
    fn pages_whose_builds_all_fail_fail_rather_than_wait() {
        let pool = std::sync::Arc::new(SessionPool::<u32>::new(2));
        let both_building = std::sync::Arc::new(std::sync::Barrier::new(2));
        let (sender, receiver) = std::sync::mpsc::channel();
        for _ in 0..2 {
            let (pool, both_building, sender) =
                (pool.clone(), both_building.clone(), sender.clone());
            std::thread::spawn(move || {
                let first = std::sync::atomic::AtomicBool::new(true);
                let build = || {
                    // Only the first builds race; a retry fails on its own.
                    if first.swap(false, std::sync::atomic::Ordering::SeqCst) {
                        both_building.wait();
                    }
                    Err(ExtractionError::resource_limit("out of memory"))
                };
                let outcome = pool.acquire(build, &CancellationToken::new()).map(|_| ());
                let _ = sender.send(outcome.is_err());
            });
        }
        for _ in 0..2 {
            let failed = receiver
                .recv_timeout(Duration::from_secs(10))
                .expect("a page waited for a set no one was building");
            assert!(failed);
        }
        assert_eq!(pool.lock().created, 0);
    }
}
