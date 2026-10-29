//! Which local model reads a document.
//!
//! Intern pins two local models (see [`crate::manifest::ModelTier`]): the
//! accurate one, Qwen3.5 2B, and an optional quick one, Bonsai 2 2B, about two
//! thirds of the accurate one's time and two thirds of its memory. On the
//! 77-document evaluation (`bench/reports/bonsai-linux-cpu-2026-10-27`) the
//! quick model named 37 documents correctly to Qwen's 52-54, and marked 23
//! wrong names ready, so a quick reading is never ready: it always goes to a
//! person ([`hold_fast_reading`]), and is read again by the accurate model
//! once it can be. Everything here is a pure decision over what is installed
//! and how much memory is free, so every policy - and every threshold the
//! benchmarks move - is one named constant or one function, tested on its own.
//!
//! The runtime asks [`plan`] before every document, so a setting changed in
//! Settings, a model that finished downloading, or memory that came back takes
//! effect at the next document with nothing restarted.

use serde::{Deserialize, Serialize};

use crate::domain::{DocumentAnalysis, ProposalStatus, ReviewReason};
use crate::manifest::ModelTier;
use crate::server::ServerOptions;

/// What a person asked for in Settings.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelPreference {
    /// Intern chooses: the policy [`AUTOMATIC_STRATEGY`] names, within the
    /// memory the machine has free.
    #[default]
    Automatic,
    /// The accurate model whenever it is installed and memory allows; the
    /// fast model stands in, read again later, only while the machine is
    /// short of memory.
    Accurate,
    /// The fast model whenever it is installed. Its readings still always go
    /// to review: asking for speed does not make them safer to file.
    Fast,
}

impl ModelPreference {
    pub const ALL: [Self; 3] = [Self::Automatic, Self::Accurate, Self::Fast];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Accurate => "accurate",
            Self::Fast => "fast",
        }
    }
}

/// How [`ModelPreference::Automatic`] spends the two models.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomaticStrategy {
    /// Every document is read by the accurate model when it is installed and
    /// fits in memory, and by the fast one otherwise.
    AccurateFirst,
    /// Every document is read by the fast model first, and read again by the
    /// accurate one only when the fast reading carries a signal
    /// [`escalation`] recognizes. Cheaper on most documents; only as good as
    /// the signals. Kept, tested, and off: the 77-document evaluation found
    /// the quick model's review flag a poor trigger - the documents it sends
    /// to review are mostly ones a bigger model also gets wrong, and where the
    /// bigger model helps is on readings the quick one marked ready.
    Cascade,
}

/// The strategy Automatic uses: the accurate model for everything it can
/// read, the fast one only as a stand-in. See [`AutomaticStrategy::Cascade`]
/// for why the cascade is off.
pub const AUTOMATIC_STRATEGY: AutomaticStrategy = AutomaticStrategy::AccurateFirst;

const MIB: u64 = 1024 * 1024;

/// The parser worker's peak while it reads a document: 0.71 GB on a 25-page
/// scan through PP-OCR, the most any InternBench document needed. It runs
/// beside the model, so a model is judged to fit only with room for it too.
pub const WORKER_PEAK: u64 = 680 * MIB;

/// What is left for the person's own work once a model and the worker are
/// loaded: Intern runs in the background, and the window in front must not
/// start paging because of it.
pub const HEADROOM: u64 = 1_024 * MIB;

/// Memory that must be available - free, or reclaimable without paging -
/// before the fast model is loaded: its server, the worker, and the
/// headroom. It is the least the machine can be asked for, so it decides
/// nothing yet; it is here so the fast model's cost is said in one place.
pub const MIN_AVAILABLE_FOR_FAST: u64 = 3_200 * MIB;

/// Memory that must be available before the accurate model is loaded: its
/// server, the worker, and the headroom, rounded up to 4 GiB. A 14 GB laptop
/// with a browser and an office suite open usually has twice that.
pub const MIN_AVAILABLE_FOR_ACCURATE: u64 = 4_096 * MIB;

/// The memory a tier needs available before it is loaded.
pub const fn min_available_for(tier: ModelTier) -> u64 {
    match tier {
        ModelTier::Fast => MIN_AVAILABLE_FOR_FAST,
        ModelTier::Accurate => MIN_AVAILABLE_FOR_ACCURATE,
    }
}

/// About what a loaded tier's server holds, with `--cache-ram 0` (see
/// [`crate::server::LlamaServer::arguments_for`]), from the 77-document
/// evaluation's measured resident sets on the AVX-512 CPU variant, the larger
/// of the two a laptop may load (the AVX2 one repacks less):
///
/// * Bonsai 2 2B: 1,418 MiB after twelve real requests (0.9 GB on AVX2).
/// * Qwen3.5 2B: 2,175-2,185 MiB after load and warm-up in every InternBench
///   chunk; with the prompt cache off it stops growing from there, as the
///   Bonsai servers do (2B: about 4 MB a document instead of 80).
///
/// Unloading the model to load another gives this back, so the runtime
/// counts it as available when it plans with a model already loaded:
/// otherwise a loaded accurate model would be judged not to fit beside
/// itself, and swapped for the fast one before every document.
pub const fn resident_estimate(tier: ModelTier) -> u64 {
    match tier {
        ModelTier::Fast => 1_420 * MIB,
        ModelTier::Accurate => 2_200 * MIB,
    }
}

/// What unloading `tier`'s model gives back to [`available_memory_bytes`],
/// for a model whose weights file is `model_file_bytes` long.
///
/// Not the whole [`resident_estimate`]: llama-server maps the GGUF, and the
/// mapped weights are clean file-backed pages that `MemAvailable` on Linux,
/// and the available physical memory on Windows (which counts the standby
/// list), already report as reclaimable. Crediting them again counted them
/// twice, so a loaded quick model could make the accurate one look as if it
/// fit on a machine that cannot hold it. Only the rest - the context, the
/// compute buffers, any repacked weights - comes back on unload.
pub const fn unload_gives_back(tier: ModelTier, model_file_bytes: u64) -> u64 {
    resident_estimate(tier).saturating_sub(model_file_bytes)
}

/// llama-server flags for one tier alone, added after the ones every launch
/// sets ([`crate::server::LlamaServer::arguments_for`]). Empty until the
/// benchmarks show a flag that helps one model and not the other: this is the
/// one place to add it.
pub const fn tier_server_flags(tier: ModelTier) -> &'static [&'static str] {
    match tier {
        ModelTier::Fast | ModelTier::Accurate => &[],
    }
}

/// How the server for `tier` is launched: the common options, and the tier's
/// own flags.
pub fn server_options(tier: ModelTier) -> ServerOptions {
    ServerOptions {
        extra_arguments: tier_server_flags(tier)
            .iter()
            .map(|flag| (*flag).to_owned())
            .collect(),
        ..ServerOptions::default()
    }
}

/// How long a loaded model may sit without a document before it is unloaded,
/// giving its memory back to the machine. The next document loads it again,
/// which costs seconds; holding a gigabyte or two all day for nothing costs
/// the person working beside it far more.
pub const IDLE_UNLOAD_AFTER: std::time::Duration = std::time::Duration::from_secs(10 * 60);

/// Holds a reading the fast model made for a person: never ready, whatever
/// validation made of it, and with [`ReviewReason::FastModelReading`] among
/// its reasons so the review panel can say why. A queue that renames ready
/// documents on its own never renames one of these.
pub fn hold_fast_reading(analysis: &mut DocumentAnalysis) {
    analysis.status = ProposalStatus::NeedsReview;
    if !analysis
        .review_reasons
        .contains(&ReviewReason::FastModelReading)
    {
        analysis.review_reasons.push(ReviewReason::FastModelReading);
    }
}

/// Whether `tier` fits in `available` bytes. Memory that cannot be measured
/// is assumed to fit: refusing the better model on a machine whose memory
/// Intern cannot read would make it worse everywhere for nothing.
pub fn fits(tier: ModelTier, available: Option<u64>) -> bool {
    available.is_none_or(|available| available >= min_available_for(tier))
}

/// What reads the next document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TierPlan {
    /// The tier that reads it first.
    pub first: ModelTier,
    /// Whether a reading by `first` is read again by the accurate tier when
    /// [`escalation`] finds a reason to.
    pub escalate: bool,
    /// The reading is one the policy would not settle for: the accurate tier
    /// was wanted but is not installed yet, or did not fit in memory. A
    /// reading like that is read again once the accurate tier can be used,
    /// unless a person has acted on it first.
    pub provisional: bool,
}

/// The plan for the next document, under [`AUTOMATIC_STRATEGY`]. `installed`
/// lists the tiers that can be loaded; `available` is the memory free for a
/// model, `None` when it cannot be measured. `None` when nothing is
/// installed.
pub fn plan(
    preference: ModelPreference,
    installed: &[ModelTier],
    available: Option<u64>,
) -> Option<TierPlan> {
    plan_with(preference, AUTOMATIC_STRATEGY, installed, available)
}

/// [`plan`], with the Automatic strategy given.
pub fn plan_with(
    preference: ModelPreference,
    strategy: AutomaticStrategy,
    installed: &[ModelTier],
    available: Option<u64>,
) -> Option<TierPlan> {
    let fast = installed.contains(&ModelTier::Fast);
    let accurate = installed.contains(&ModelTier::Accurate);
    // One model installed is the only model there is, whatever was asked
    // for and whatever memory says: refusing to read at all helps nobody.
    // The fast model alone is provisional unless the fast model is what was
    // asked for.
    match (fast, accurate) {
        (false, false) => return None,
        (false, true) => return Some(settled(ModelTier::Accurate)),
        (true, false) => {
            return Some(TierPlan {
                first: ModelTier::Fast,
                escalate: false,
                provisional: preference != ModelPreference::Fast,
            });
        }
        (true, true) => {}
    }
    Some(match preference {
        ModelPreference::Fast => settled(ModelTier::Fast),
        // Asked for by name, and still within the memory guard: on a laptop
        // that is already short of memory, loading the larger model pages
        // out whatever the person is working in, and Intern runs in the
        // background so that it never does that. The fast model stands in
        // and the reading is read again once memory allows.
        ModelPreference::Accurate if fits(ModelTier::Accurate, available) => {
            settled(ModelTier::Accurate)
        }
        ModelPreference::Accurate => stand_in(),
        ModelPreference::Automatic => automatic(strategy, available),
    })
}

/// The Automatic policy with both tiers installed: the one place it lives.
fn automatic(strategy: AutomaticStrategy, available: Option<u64>) -> TierPlan {
    let accurate_fits = fits(ModelTier::Accurate, available);
    match (strategy, accurate_fits) {
        (AutomaticStrategy::AccurateFirst, true) => settled(ModelTier::Accurate),
        (AutomaticStrategy::Cascade, true) => TierPlan {
            first: ModelTier::Fast,
            escalate: true,
            provisional: false,
        },
        (_, false) => stand_in(),
    }
}

/// The accurate model would not fit beside what else is running, so the fast
/// one reads - also when it would not fit either, being the least the
/// machine can be asked for - and the reading is read again once memory
/// allows.
const fn stand_in() -> TierPlan {
    TierPlan {
        first: ModelTier::Fast,
        escalate: false,
        provisional: true,
    }
}

const fn settled(tier: ModelTier) -> TierPlan {
    TierPlan {
        first: tier,
        escalate: false,
        provisional: false,
    }
}

/// Why a fast reading was read again by the accurate model.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationReason {
    /// Validation sent the reading to review, with a reason.
    ReviewReason,
    /// The reading is not ready to apply, though validation named no reason.
    NotReady,
    /// The model was unsure of a date or party token.
    LowTokenConfidence,
}

/// The signals that send a fast reading to the accurate model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EscalationPolicy {
    /// Any review reason at all.
    pub on_review_reason: bool,
    /// A status other than ready.
    pub on_not_ready: bool,
    /// Escalate when the least probable date or party token is below this.
    /// Only a reading that carried token probabilities can trip it: see
    /// [`wants_token_confidence`].
    pub below_token_confidence: Option<f32>,
}

/// The token probability below which a fast reading is read again. A
/// placeholder until the benchmark calibrates it.
pub const ESCALATE_BELOW_TOKEN_CONFIDENCE: f32 = 0.5;

/// The escalation policy in use.
pub const ESCALATION: EscalationPolicy = EscalationPolicy {
    on_review_reason: true,
    on_not_ready: true,
    below_token_confidence: Some(ESCALATE_BELOW_TOKEN_CONFIDENCE),
};

/// Whether a fast reading should be read again by the accurate model, and
/// why, under [`ESCALATION`].
pub fn escalation(analysis: &DocumentAnalysis) -> Option<EscalationReason> {
    escalation_with(&ESCALATION, analysis)
}

/// [`escalation`] under any policy.
pub fn escalation_with(
    policy: &EscalationPolicy,
    analysis: &DocumentAnalysis,
) -> Option<EscalationReason> {
    if policy.on_review_reason && !analysis.review_reasons.is_empty() {
        return Some(EscalationReason::ReviewReason);
    }
    if policy.on_not_ready && analysis.status != ProposalStatus::Ready {
        return Some(EscalationReason::NotReady);
    }
    let unsure = policy
        .below_token_confidence
        .zip(analysis.token_confidence)
        .is_some_and(|(threshold, confidence)| confidence.min < threshold);
    unsure.then_some(EscalationReason::LowTokenConfidence)
}

/// The token probability below which an accurate reading goes to review
/// instead of being filed: the least probable token of its date or of a
/// party's name ([`crate::domain::TokenConfidence::min`]). The model naming
/// a date or a party it would rather not have written is the one signal
/// validation cannot see, since the value it chose is still in the text.
///
/// On the 77-document evaluation (Qwen3.5 2B, the packaged runtime, after
/// the party rules of October 2026) it sent 15 of the 65 ready readings to
/// review: 7 of the 11 wrong names among them, and 8 right ones. The value
/// was chosen on the fixture corpus (`docs/model-candidates.md`), not on
/// those documents.
pub const REVIEW_BELOW_TOKEN_CONFIDENCE: Option<f32> = Some(0.5);

/// The token probability below which `tier`'s readings go to review. A fast
/// reading always goes to review ([`hold_fast_reading`]), so only the
/// accurate tier has one.
pub fn review_below_token_confidence(tier: ModelTier) -> Option<f32> {
    match tier {
        ModelTier::Accurate => REVIEW_BELOW_TOKEN_CONFIDENCE,
        ModelTier::Fast => None,
    }
}

/// Whether `tier`'s requests should ask the server for token probabilities:
/// a reading the review gate judges by them, or one the cascade may escalate
/// on them. The server pays a softmax over the vocabulary for every
/// generated token (about 1-2% of a reading's time), so no other reading
/// asks.
pub fn wants_token_confidence(tier: ModelTier) -> bool {
    review_below_token_confidence(tier).is_some()
        || (matches!(AUTOMATIC_STRATEGY, AutomaticStrategy::Cascade)
            && tier == ModelTier::Fast
            && ESCALATION.below_token_confidence.is_some())
}

/// The memory available for a model to load into, in bytes, or `None` when
/// it cannot be read. Never panics.
///
/// On Linux, the kernel's own estimate (`MemAvailable`): free memory plus the
/// page cache and reclaimable slabs it could give up without swapping. On
/// Windows, `GlobalMemoryStatusEx`'s available physical memory, which counts
/// the standby list the same way.
pub fn available_memory_bytes() -> Option<u64> {
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

// One Win32 call with no safe wrapper in the dependency tree. The rest of the
// crate keeps `#![deny(unsafe_code)]`.
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
    use crate::domain::{AnalysisTelemetry, ReviewReason, TokenConfidence, ValidatedProposal};

    const GIB: u64 = 1024 * MIB;
    const BOTH: &[ModelTier] = &[ModelTier::Fast, ModelTier::Accurate];
    const FAST: &[ModelTier] = &[ModelTier::Fast];
    const ACCURATE: &[ModelTier] = &[ModelTier::Accurate];

    fn plan_of(
        preference: ModelPreference,
        strategy: AutomaticStrategy,
        installed: &[ModelTier],
        available: Option<u64>,
    ) -> (ModelTier, bool, bool) {
        let plan =
            plan_with(preference, strategy, installed, available).expect("something is installed");
        (plan.first, plan.escalate, plan.provisional)
    }

    use AutomaticStrategy::{AccurateFirst, Cascade};
    use ModelPreference::{Accurate, Automatic, Fast};
    use ModelTier::{Accurate as A, Fast as F};

    #[test]
    fn nothing_installed_is_no_plan() {
        for preference in ModelPreference::ALL {
            for strategy in [AccurateFirst, Cascade] {
                assert_eq!(plan_with(preference, strategy, &[], Some(16 * GIB)), None);
            }
        }
    }

    #[test]
    fn a_named_tier_is_used_whenever_it_is_installed_and_fits() {
        for strategy in [AccurateFirst, Cascade] {
            for available in [None, Some(MIN_AVAILABLE_FOR_ACCURATE), Some(16 * GIB)] {
                assert_eq!(
                    plan_of(Accurate, strategy, BOTH, available),
                    (A, false, false)
                );
            }
            // The fast model, asked for by name, reads whatever the memory:
            // there is nothing smaller to fall back to.
            for available in [None, Some(0), Some(GIB), Some(16 * GIB)] {
                assert_eq!(plan_of(Fast, strategy, BOTH, available), (F, false, false));
            }
        }
    }

    /// Most accurate is still a background reader: short of memory, the fast
    /// model stands in and the reading is read again when memory allows,
    /// rather than paging out what the person is working in.
    #[test]
    fn the_accurate_model_asked_for_by_name_keeps_to_the_memory_guard() {
        for strategy in [AccurateFirst, Cascade] {
            for available in [Some(MIN_AVAILABLE_FOR_ACCURATE - 1), Some(GIB), Some(0)] {
                assert_eq!(
                    plan_of(Accurate, strategy, BOTH, available),
                    (F, false, true)
                );
            }
        }
    }

    /// The accurate model was asked for - by name, or by Automatic - and is
    /// not there yet: the fast one reads, and its reading is provisional.
    /// The fast model asked for by name is no stand-in for anything.
    #[test]
    fn the_fast_model_alone_stands_in_provisionally_unless_it_was_asked_for() {
        for strategy in [AccurateFirst, Cascade] {
            for available in [None, Some(GIB), Some(16 * GIB)] {
                assert_eq!(
                    plan_of(Accurate, strategy, FAST, available),
                    (F, false, true)
                );
                assert_eq!(
                    plan_of(Automatic, strategy, FAST, available),
                    (F, false, true)
                );
                assert_eq!(plan_of(Fast, strategy, FAST, available), (F, false, false));
            }
        }
    }

    /// Only the accurate model installed - chosen from disk on its own - is
    /// the only model there is, and it is used for everything.
    #[test]
    fn the_accurate_model_alone_reads_everything() {
        for preference in ModelPreference::ALL {
            for strategy in [AccurateFirst, Cascade] {
                for available in [None, Some(0), Some(16 * GIB)] {
                    assert_eq!(
                        plan_of(preference, strategy, ACCURATE, available),
                        (A, false, false)
                    );
                }
            }
        }
    }

    #[test]
    fn automatic_accurate_first_reads_with_the_accurate_model_when_it_fits() {
        for available in [None, Some(MIN_AVAILABLE_FOR_ACCURATE), Some(16 * GIB)] {
            assert_eq!(
                plan_of(Automatic, AccurateFirst, BOTH, available),
                (A, false, false)
            );
        }
        // Just short of fitting, and far short: the fast model stands in.
        for available in [
            Some(MIN_AVAILABLE_FOR_ACCURATE - 1),
            Some(MIN_AVAILABLE_FOR_FAST),
            Some(0),
        ] {
            assert_eq!(
                plan_of(Automatic, AccurateFirst, BOTH, available),
                (F, false, true)
            );
        }
    }

    #[test]
    fn automatic_cascade_reads_fast_and_escalates_only_when_the_accurate_model_fits() {
        for available in [None, Some(MIN_AVAILABLE_FOR_ACCURATE), Some(16 * GIB)] {
            assert_eq!(
                plan_of(Automatic, Cascade, BOTH, available),
                (F, true, false)
            );
        }
        for available in [Some(MIN_AVAILABLE_FOR_ACCURATE - 1), Some(0)] {
            assert_eq!(
                plan_of(Automatic, Cascade, BOTH, available),
                (F, false, true)
            );
        }
    }

    #[test]
    fn the_default_strategy_is_the_one_plan_uses() {
        assert_eq!(AUTOMATIC_STRATEGY, AccurateFirst);
        for preference in ModelPreference::ALL {
            for installed in [BOTH, FAST, ACCURATE, &[]] {
                for available in [None, Some(GIB), Some(16 * GIB)] {
                    assert_eq!(
                        plan(preference, installed, available),
                        plan_with(preference, AUTOMATIC_STRATEGY, installed, available)
                    );
                }
            }
        }
        assert_eq!(ModelPreference::default(), Automatic);
    }

    #[test]
    fn fitting_is_a_floor_and_unknown_memory_fits() {
        assert!(fits(A, None));
        assert!(fits(F, None));
        assert!(fits(A, Some(MIN_AVAILABLE_FOR_ACCURATE)));
        assert!(!fits(A, Some(MIN_AVAILABLE_FOR_ACCURATE - 1)));
        assert!(fits(F, Some(MIN_AVAILABLE_FOR_FAST)));
        assert!(!fits(F, Some(MIN_AVAILABLE_FOR_FAST - 1)));
        assert_eq!(min_available_for(F), MIN_AVAILABLE_FOR_FAST);
        assert_eq!(min_available_for(A), MIN_AVAILABLE_FOR_ACCURATE);
        // The guard asks for more than either model holds, and more for the
        // larger model.
        for tier in ModelTier::ALL {
            assert!(min_available_for(tier) > resident_estimate(tier));
        }
        assert!(min_available_for(A) > min_available_for(F));
    }

    #[test]
    fn a_preference_reads_and_writes_as_settings_store_it() {
        for preference in ModelPreference::ALL {
            let json = serde_json::to_value(preference).unwrap();
            assert_eq!(json, serde_json::Value::String(preference.as_str().into()));
            assert_eq!(
                serde_json::from_value::<ModelPreference>(json).unwrap(),
                preference
            );
        }
        assert!(serde_json::from_value::<ModelPreference>("best".into()).is_err());
    }

    fn reading(
        status: ProposalStatus,
        reasons: &[ReviewReason],
        confidence: Option<f32>,
    ) -> DocumentAnalysis {
        DocumentAnalysis {
            filename: "2026-01-16 Dispatch Note.pdf".into(),
            description: String::new(),
            status,
            review_reasons: reasons.to_vec(),
            proposal: ValidatedProposal::default(),
            telemetry: AnalysisTelemetry::default(),
            model_proposal: None,
            stated_dates: Vec::new(),
            text_fingerprint: None,
            token_confidence: confidence.map(|min| TokenConfidence {
                min,
                mean: 0.9,
                tokens: 12,
            }),
            facts: None,
        }
    }

    /// A fast reading is never ready, whatever validation made of it, and
    /// says why exactly once, beside any reason it already had.
    #[test]
    fn a_fast_reading_always_goes_to_review() {
        let mut ready = reading(ProposalStatus::Ready, &[], Some(0.99));
        hold_fast_reading(&mut ready);
        assert_eq!(ready.status, ProposalStatus::NeedsReview);
        assert_eq!(ready.review_reasons, [ReviewReason::FastModelReading]);

        let mut doubtful = reading(
            ProposalStatus::NeedsReview,
            &[ReviewReason::DateMissing],
            None,
        );
        hold_fast_reading(&mut doubtful);
        hold_fast_reading(&mut doubtful);
        assert_eq!(
            doubtful.review_reasons,
            [ReviewReason::DateMissing, ReviewReason::FastModelReading]
        );
        assert_eq!(
            ReviewReason::FastModelReading.as_str(),
            "FAST_MODEL_READING"
        );
    }

    /// The memory guard is the model, the worker beside it and the headroom
    /// the person keeps, and the accurate model needs more of it.
    #[test]
    fn each_guard_covers_the_model_the_worker_and_the_headroom() {
        for tier in ModelTier::ALL {
            assert!(min_available_for(tier) >= resident_estimate(tier) + WORKER_PEAK + HEADROOM);
        }
    }

    /// The mapped weights are counted as available already, so only the
    /// rest of a loaded model comes back on unload - never less than nothing.
    #[test]
    fn unloading_gives_back_only_what_is_not_the_mapped_file() {
        assert_eq!(unload_gives_back(F, 0), resident_estimate(F));
        assert_eq!(
            unload_gives_back(A, 1_280_835_840),
            resident_estimate(A) - 1_280_835_840
        );
        assert_eq!(unload_gives_back(F, u64::MAX), 0);
    }

    #[test]
    fn a_ready_and_sure_reading_is_not_escalated() {
        assert_eq!(escalation(&reading(ProposalStatus::Ready, &[], None)), None);
        assert_eq!(
            escalation(&reading(ProposalStatus::Ready, &[], Some(0.99))),
            None
        );
        assert_eq!(
            escalation(&reading(
                ProposalStatus::Ready,
                &[],
                Some(ESCALATE_BELOW_TOKEN_CONFIDENCE)
            )),
            None,
            "the threshold itself is sure enough"
        );
    }

    #[test]
    fn review_an_unready_status_and_low_confidence_each_escalate() {
        assert_eq!(
            escalation(&reading(
                ProposalStatus::NeedsReview,
                &[ReviewReason::DateMissing],
                Some(0.99)
            )),
            Some(EscalationReason::ReviewReason)
        );
        assert_eq!(
            escalation(&reading(ProposalStatus::NeedsReview, &[], None)),
            Some(EscalationReason::NotReady)
        );
        assert_eq!(
            escalation(&reading(ProposalStatus::Ready, &[], Some(0.2))),
            Some(EscalationReason::LowTokenConfidence)
        );
    }

    /// Every signal is a switch, so tuning the policy is changing a value.
    #[test]
    fn each_signal_can_be_switched_off() {
        let none = EscalationPolicy {
            on_review_reason: false,
            on_not_ready: false,
            below_token_confidence: None,
        };
        for analysis in [
            reading(
                ProposalStatus::NeedsReview,
                &[ReviewReason::TypeMissing],
                Some(0.0),
            ),
            reading(ProposalStatus::NeedsReview, &[], Some(0.0)),
            reading(ProposalStatus::Ready, &[], Some(0.0)),
        ] {
            assert_eq!(escalation_with(&none, &analysis), None);
        }
        let only_confidence = EscalationPolicy {
            below_token_confidence: Some(0.9),
            ..none
        };
        assert_eq!(
            escalation_with(
                &only_confidence,
                &reading(
                    ProposalStatus::NeedsReview,
                    &[ReviewReason::TypeMissing],
                    Some(0.8)
                )
            ),
            Some(EscalationReason::LowTokenConfidence)
        );
    }

    #[test]
    fn token_probabilities_are_asked_for_only_where_something_reads_them() {
        assert_eq!(review_below_token_confidence(A), Some(0.5));
        assert_eq!(review_below_token_confidence(F), None);
        assert!(wants_token_confidence(A), "the review gate reads them");
        assert_eq!(
            wants_token_confidence(F),
            AUTOMATIC_STRATEGY == Cascade,
            "a fast reading is always reviewed, and AccurateFirst never escalates"
        );
    }

    #[test]
    fn each_tier_is_launched_with_the_common_options_and_its_own_flags() {
        for tier in ModelTier::ALL {
            let options = server_options(tier);
            assert_eq!(
                ServerOptions {
                    extra_arguments: Vec::new(),
                    ..options.clone()
                },
                ServerOptions::default()
            );
            assert_eq!(options.extra_arguments, tier_server_flags(tier));
        }
        assert_eq!(IDLE_UNLOAD_AFTER.as_secs(), 600);
    }

    #[test]
    fn mem_available_is_read_from_meminfo() {
        let meminfo = "MemTotal:       16303316 kB\nMemFree:          981236 kB\n\
                       MemAvailable:   13631488 kB\nBuffers:          123 kB\n";
        assert_eq!(mem_available_from_meminfo(meminfo), Some(13_631_488 * 1024));
        assert_eq!(mem_available_from_meminfo("MemAvailable: 42\n"), Some(42));
        for broken in [
            "",
            "MemTotal: 1 kB\n",
            "MemAvailable: lots kB\n",
            "MemAvailable: 12 MB\n",
            "MemAvailable:\n",
            "MemAvailable: 99999999999999999999 kB\n",
        ] {
            assert_eq!(mem_available_from_meminfo(broken), None, "{broken:?}");
        }
        assert_eq!(
            mem_available_from_meminfo("MemAvailable: 18446744073709551615 kB\n"),
            None,
            "overflow is unknown, not a wrapped number"
        );
    }

    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn this_machine_reports_some_available_memory() {
        let available = available_memory_bytes().expect("readable here");
        assert!(available > 0);
    }
}
