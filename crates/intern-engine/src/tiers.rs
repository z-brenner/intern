//! Which local model reads a document.
//!
//! Intern pins two local models (see [`crate::manifest::ModelTier`]): the
//! accurate one, Qwen3.5 9B, and a quick one, Qwen3.5 4B, about half the
//! accurate one's time and little over half its memory. Both are asked the
//! same way - the same prompt, the same validation - but on the real-document
//! sets (`bench/reports/model-qualification-2026-11`) the 4B filed a wrong
//! name for every right one at any review gate, so a quick reading is never
//! ready: it always goes to a person ([`hold_fast_reading`]), and is read
//! again by the accurate model once it can be. What sets the two apart is
//! memory: the accurate model needs more of it than a 16 GB laptop with a
//! browser and an office suite open always has, so the quick one stands in
//! while memory is short. Everything
//! here is a pure decision over what is installed and how much memory is
//! free, so every policy - and every threshold the benchmarks move - is one
//! named constant or one function, tested on its own.
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
    /// The fast model whenever it is installed, with nothing read again by
    /// the accurate one.
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
///
/// One page at a time. Two pages at once took the worker's peak to 1.4 GB
/// over all of InternBench (docs/ocr.md), so the worker reads a second page
/// at once only with the memory for it available as a document's pages
/// start to be read (`intern_worker::paddle::pages_at_once`): the two-page
/// peak and [`HEADROOM`], and on Linux, where [`available_memory_bytes`]
/// counts them, the weights a loaded accurate model keeps reading - 8 GiB
/// on Linux, 3 GiB on Windows. Near a floor it reads one page at a time,
/// which is what this counts.
pub const WORKER_PEAK: u64 = 680 * MIB;

/// What is left for the person's own work once a model and the worker are
/// loaded: Intern runs in the background, and the window in front must not
/// start paging because of it.
pub const HEADROOM: u64 = 1_024 * MIB;

/// Memory that must be available - free, or reclaimable without paging -
/// before the fast model is loaded: its server, the worker, and the
/// headroom, rounded up to 6.5 GiB. It is the least the machine can be asked
/// for, so it decides nothing yet; it is here so the fast model's cost is
/// said in one place.
pub const MIN_AVAILABLE_FOR_FAST: u64 = 6_656 * MIB;

/// Memory that must be available before the accurate model is loaded: what
/// its server holds under pressure ([`working_set`]), the worker, and the
/// headroom, rounded up to 8 GiB. A 16 GB laptop has that with a browser
/// open; not with a browser and an office suite at work; then the fast
/// model stands in. Until alpha.13 this was 10 GiB, the server's resident
/// high-water mark plus the rest, before the mapped weights were measured
/// under a memory limit (`bench/reports/speed-linux-cpu-2026-11-05`).
pub const MIN_AVAILABLE_FOR_ACCURATE: u64 = 8_192 * MIB;

/// The memory a tier needs available before it is loaded.
pub const fn min_available_for(tier: ModelTier) -> u64 {
    match tier {
        ModelTier::Fast => MIN_AVAILABLE_FOR_FAST,
        ModelTier::Accurate => MIN_AVAILABLE_FOR_ACCURATE,
    }
}

/// About what a loaded tier's server holds, with `--cache-ram 0` (see
/// [`crate::server::LlamaServer::arguments_for`]): the resident set's high
/// water mark over a whole live run of the 77-document evaluation and the
/// real-document sets, almost all of it the mapped weights, on the AVX-512
/// CPU variant, the larger of the two a laptop may load (the AVX2 one
/// repacks less):
///
/// * Qwen3.5 4B: 4,600-4,700 MiB (2.7 GB of weights).
/// * Qwen3.5 9B: 8,300-8,500 MiB (5.7 GB of weights).
///
/// With the prompt cache off neither grows from there. Unloading the model
/// to load another gives back what of this the machine does not already
/// count as available, so the runtime counts that as available when it plans
/// with a model already loaded: otherwise a loaded accurate model would be
/// judged not to fit beside itself, and swapped for the fast one before
/// every document. It reads that from the live server
/// ([`memory_released_by`]); this estimate is for when it cannot
/// ([`unload_gives_back`]).
///
/// Measured with repacked weights. With `--no-repack` ([`tier_server_flags`])
/// the 9B's high-water mark is 6,133-6,184 MiB, so on Linux the estimate now
/// credits about 2 GiB more than unloading gives back. It is kept: it stands
/// in only when the live server cannot be read, and on Windows, where it is
/// already short, lowering it to that would have the runner with a browser's
/// 2 GiB held swap the models before every document (11,570 MiB available
/// before any load, run 43063958237).
pub const fn resident_estimate(tier: ModelTier) -> u64 {
    match tier {
        ModelTier::Fast => 4_700 * MIB,
        ModelTier::Accurate => 8_300 * MIB,
    }
}

/// What a loaded tier's server must keep in memory to read at its speed.
/// With repacked weights that was less than [`resident_estimate`], because
/// the mapped weights the CPU backend repacks are read once and never again,
/// and the system reclaims those pages first. Measured for the 9B under a
/// memory limit on Linux (`bench/reports/speed-linux-cpu-2026-11-05`), at a
/// 6 GB limit: 3,317 MiB of anonymous memory and 2,726 MiB of weights it
/// kept reading, 6,043 MiB in all (the report's "6.2 GB"). There it read a
/// document in 95 s against 72 s with no limit; at 5 GB it thrashed
/// (421 s). With `--no-repack` ([`tier_server_flags`]) it reads the
/// originals in the mapped file in place of the copy: all of the file but
/// the token embeddings, which are read a row at a time. About 4,880 MiB of
/// the 5,417 MiB file stayed in its working set on Windows (run
/// 43063958237); its peak resident set on Linux, 6,133-6,184 MiB
/// (`bench/reports/speed-linux-cpu-2026-11-11`), is a high-water mark of
/// whatever was ever resident. Both are within this figure; what it needs
/// was not measured under a limit. The 4B was not measured under a limit;
/// its resident set stands in. On Windows the 9B's working set right after
/// a document on a 16 GiB runner was 8,110 MiB with repacked weights, the
/// touched originals included (`bench/reports/windows-memory-2026-11-09`),
/// and 5,490-5,491 MiB without (run 43063958237; 5,229-5,491 MiB from load
/// to a minute idle); what it needs under a squeeze there is not measured.
pub const fn working_set(tier: ModelTier) -> u64 {
    match tier {
        ModelTier::Fast => 4_700 * MIB,
        ModelTier::Accurate => 6_300 * MIB,
    }
}

/// An estimate of what unloading `tier`'s model gives back to
/// [`available_memory_bytes`], for a model whose weights file is
/// `model_file_bytes` long: what the runtime credits a loaded model with
/// when the live server's own figure ([`memory_released_by`]) cannot be
/// read.
///
/// Not the whole [`resident_estimate`]: llama-server maps the GGUF, and on
/// Linux the mapped weights are page cache that `MemAvailable` already
/// reports as reclaimable. Crediting them again counted them twice, so a
/// loaded quick model could make the accurate one look as if it fit on a
/// machine that cannot hold it. Only the rest - the context, the compute
/// buffers, any repacked weights - comes back on unload: 2,883 MiB for the
/// 9B against 3,317 MiB of anonymous memory measured with repacked weights
/// (`bench/reports/speed-linux-cpu-2026-11-05`). The 9B runs with
/// `--no-repack` from alpha.17, and its peak resident set less its file is
/// about 0.7 GiB (`bench/reports/speed-linux-cpu-2026-11-11`), so on Linux
/// this now credits it about 2 GiB too much (see [`resident_estimate`]).
///
/// On Windows it is far short. The available physical memory counts the
/// standby list, where a mapped file's cached pages wait, but not the pages
/// a process has in its working set, and the server keeps the weights it
/// has touched there until it exits or the system trims them. Measured on a
/// 16 GiB runner (`bench/reports/windows-memory-2026-11-09`): right after a
/// document the 9B's working set was 8,107-8,110 MiB with repacked weights
/// (5,490-5,491 MiB with `--no-repack`, run 43063958237) and the 4B's
/// 4,513-4,514, against the 2,883 and 2,086 this credits. With 2 GiB held
/// by another process, the plan's arithmetic on the repacked numbers stands
/// the loaded 9B down for the 4B before the next document, and the loaded
/// 4B down for the 9B before the one after.
pub const fn unload_gives_back(tier: ModelTier, model_file_bytes: u64) -> u64 {
    resident_estimate(tier).saturating_sub(model_file_bytes)
}

/// How much more than [`MIN_AVAILABLE_FOR_ACCURATE`] the plan asks for
/// before it stands a loaded fast model down for the accurate one.
///
/// Credited what stopping it gives back, a loaded model leaves the plan
/// judging the machine about as it would be with nothing loaded, but not
/// exactly, and not alike for the two models. Where the 4B's figure is the
/// higher, a machine with the 9B loaded and near the floor is judged to
/// have room once the 4B is loaded, and the two swap, the 9B each time onto
/// a machine that cannot hold it: before every document of a queue, or at
/// every pause. On 16 GiB Windows runners with 2 GiB held by another
/// process, the plan's figure against the memory available before that
/// model was loaded came out:
///
/// * with repacked weights (`bench/reports/windows-memory-2026-11-09`, pass
///   3): +181 MiB with the 9B right after a document, +585 with the 4B
///   right after a document and +1,098 a minute idle, so the 4B's was the
///   higher by 404 and 917 MiB;
/// * with the 9B's `--no-repack` (`runtime-speed.yml` run 43063958237):
///   -141 MiB with the 9B right after a document, +768 with the 4B right
///   after a document and +1,453 a minute idle, so the 4B's was the higher
///   by 909 and 1,594 MiB. At 1,024 MiB, the margin from alpha.15 to
///   alpha.16, a machine with between about 7,763 and 8,333 MiB available
///   before any load would have the models swap at every pause.
///
/// The margin is the largest gap measured with room to spare, as 1,024 was
/// for 917: carried to every machine as the report's bands are, either
/// run's numbers bring the loaded 4B back only where the 9B would stay
/// (the test `a_loaded_fast_model_makes_way_only_with_room_to_spare`).
/// In both runs the 4B was probed second, after the 9B, and more memory was
/// available 3 s after its exit than before its load (1,181 and 1,449 MiB),
/// its own working set flat or falling over its idle minute while the
/// available memory rose by 581 and 685 MiB: part of its figure is the rest
/// of the machine releasing memory, so the margin errs toward keeping the
/// smaller model. The loaded 9B is still stood down at the floor itself:
/// the margin keeps the smaller model longer, never the larger one. It also
/// stops a machine whose memory drifts around the floor from swapping the
/// models back and forth. The cost: a loaded 4B, its readings provisional,
/// stays until the plan's figure reaches the floor and this margin.
pub const UPGRADE_MARGIN: u64 = 2_048 * MIB;

/// The memory the plan judges the machine by: `available`, what the
/// machine reports, plus, when a model is loaded, what stopping its server
/// would give back (`loaded`: the tier and that amount), so the machine is
/// judged as it would be once the model is unloaded; less
/// [`UPGRADE_MARGIN`] while the fast model is the one loaded.
pub const fn memory_for_plan(available: u64, loaded: Option<(ModelTier, u64)>) -> u64 {
    match loaded {
        None => available,
        Some((ModelTier::Accurate, released)) => available.saturating_add(released),
        Some((ModelTier::Fast, released)) => available
            .saturating_add(released)
            .saturating_sub(UPGRADE_MARGIN),
    }
}

/// llama-server flags for one tier alone, added after the ones every launch
/// sets ([`crate::server::LlamaServer::arguments_for`]): the one place a flag
/// that was measured on one model and not the other goes.
///
/// The accurate model runs with `--no-repack`. llama.cpp otherwise copies
/// every Q4_K weight matrix into an interleaved layout for its older x86
/// GEMM, and that copy is about 2.6 GiB of the 9B's private memory; without
/// it the weights stay mapped from the file and b11946's tiled k-quant kernel
/// reads them. Measured on the 9B with the recorded requests
/// (`bench/reports/speed-linux-cpu-2026-11-11`), each runner its own
/// baseline: the server's peak resident memory (`VmHWM`) fell from
/// 8,750-8,800 MiB to 6,133-6,184 MiB on every set, and on a 16 GiB Windows
/// runner its working set right after a document from 8,110 MiB to 5,491
/// MiB (`runtime-speed.yml` run 43063958237); time per request fell 22-27% on AMD Zen 4 and Zen 5 runners
/// (AVX-512), and 10-15% with ggml's AVX2 backend pinned, the code path most
/// laptops load; on an Intel Xeon with AMX it was unchanged. Live on
/// InternBench and the five real sets the names it gave differed from the
/// repacked run's no more than two repacked runs on different processors
/// differ from each other.
///
/// The fast model runs with it from alpha.18. On an AMD EPYC 7763 (AVX2)
/// the 4B took 8.6% less time per recorded request (58.2 s to 53.2 s,
/// `speed-battery.yml` run 43257828551), 4 of 13 replies changing as they
/// do between processors.
pub const fn tier_server_flags(tier: ModelTier) -> &'static [&'static str] {
    match tier {
        ModelTier::Fast => &["--no-repack"],
        ModelTier::Accurate => &["--no-repack"],
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

/// The folder, inside the sidecar folder, that holds the llama.cpp runtime
/// both tiers' server is started from: ggml-org's llama.cpp `b11946`, its
/// llama-server beside every library it loads and one CPU backend per x86-64
/// generation. It read the 77-document evaluation 19% faster than the
/// llama.cpp fork Intern also shipped until alpha.14, at the same accuracy
/// (`bench/reports/speed-linux-cpu-2026-10-29`), and 25% faster on Windows.
/// The folder keeps the name it had beside that fork, so an update replaces
/// its files in place.
pub const UPSTREAM_RUNTIME_FOLDER: &str = "llama-upstream";

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

/// The token probability below which the accurate model's reading goes to
/// review instead of being filed: the least probable token of its date or
/// of a party's name ([`crate::domain::TokenConfidence::min`]). The model
/// naming a date or a party it would rather not have written is the one
/// signal validation cannot see, since the value it chose is still in the
/// text.
///
/// Calibrated by replay on the four real-document sets
/// (`bench/reports/model-qualification-2026-11`): a larger model is surer of
/// its wrong names than Qwen3.5 2B was, and the 2B's 0.5 let the 9B file
/// 16 wrong names of 42 filed. At 0.6 it filed 9 wrong of 28; at 0.7, 2 of
/// 15; at 0.8, 1 of 7. It was 0.7 through alpha.16.
///
/// 0.8 from alpha.17, by a rule written down before the sixth real set was
/// read (`bench/reports/review-gate-2026-11`): of the policies that filed no
/// more wrong names than 0.7 on the development recordings, those that filed
/// the fewest wrong names on that unseen set. Three tied there, all at 0.8,
/// with the same right names too: this threshold alone, and with either of
/// two extra rules, which filed nothing more on that set. The plain
/// threshold was taken over two tied policies with an extra rule, a choice
/// the pre-registered rule left open. Replayed through this release's
/// engine, 0.7 filed 4 right names and 2 wrong ones there, 0.8 3 and 1,
/// counting the names the set scores, as the rule does; each also filed a
/// blank invoice template under the date its TODAY() cell held, which the
/// set's gold forbids. On the five development sets and InternBench 0.7
/// filed 55 right and 2 wrong, 0.8 39 and none. A name filed wrong is
/// renamed with no one looking; a right one held is a click. 51 documents
/// cannot certify a rate, and the counts say so.
pub const REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE: f32 = 0.8;

/// The same for the quick model's readings. A quick reading always goes to
/// a person ([`hold_fast_reading`]), so this decides only whether a cascade
/// sends it on to the accurate model ([`escalation_with`]): the 4B filed
/// 14 wrong names of 25 at 0.6 and 5 of 12 at 0.7 on the real sets, which
/// is why it is held.
pub const REVIEW_BELOW_TOKEN_CONFIDENCE_FAST: f32 = 0.7;

/// The token probability below which `tier`'s readings go to review, or, for
/// a quick reading, which is held anyway, to the accurate model.
pub fn review_below_token_confidence(tier: ModelTier) -> Option<f32> {
    match tier {
        ModelTier::Accurate => Some(REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE),
        ModelTier::Fast => Some(REVIEW_BELOW_TOKEN_CONFIDENCE_FAST),
    }
}

/// Whether `tier`'s requests should ask the server for token probabilities:
/// a reading the review gate judges by them, or one the cascade may escalate
/// on them. The server pays a softmax over the vocabulary for every
/// generated token (about 1-2% of a reading's time); every reading is
/// judged by the gate, so every request asks.
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
/// page cache and reclaimable slabs it could give up without swapping, mapped
/// or not. On Windows, `GlobalMemoryStatusEx`'s available physical memory: the
/// free, zero and standby lists. That counts a file's cached pages too, but
/// not the ones a process has in its working set, a running model's mapped
/// weights among them, so the two do not count a loaded model alike (see
/// [`memory_released_by`]).
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

/// What stopping `process` would add to [`available_memory_bytes`], in
/// bytes, read from the live process; `None` when it cannot be read. Never
/// panics.
///
/// On Windows, the process's working set (`GetProcessMemoryInfo`): nearly
/// every page in it is on none of the lists the available physical memory
/// counts, and goes back to one when the process exits - its private memory
/// to the free list, the mapped weights it touched to the standby list. Not
/// quite all: pages of system libraries that other processes map too are in
/// the working set and stay where they are, and the server's own pages that
/// the system has trimmed to the modified list are not in it, are not
/// counted available, and are freed at the exit all the same. On a 16 GiB
/// runner the available memory 3 s after the 9B or the 4B exited was the
/// available memory before plus the working set before, to within 83 MiB in
/// six stops (`bench/reports/windows-memory-2026-11-09`); under a squeeze,
/// when the modified list holds more of the server, the figure could run
/// short, which is not measured. On Linux, its anonymous
/// resident memory (`RssAnon`): `MemAvailable` already counts the mapped
/// weights, which are page cache, so only the rest comes back.
pub fn memory_released_by(process: &std::process::Child) -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string(format!("/proc/{}/status", process.id()))
            .ok()
            .as_deref()
            .and_then(rss_anon_from_status)
    }
    #[cfg(windows)]
    {
        windows_memory::working_set(process)
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = process;
        None
    }
}

/// `MemAvailable` from the text of `/proc/meminfo`, in bytes.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn mem_available_from_meminfo(meminfo: &str) -> Option<u64> {
    proc_field_bytes(meminfo, "MemAvailable:")
}

/// `RssAnon` from the text of `/proc/<pid>/status`, in bytes. A process
/// that has exited and not yet been reaped has no such line.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn rss_anon_from_status(status: &str) -> Option<u64> {
    proc_field_bytes(status, "RssAnon:")
}

/// The amount on the line of a `/proc` file that starts with `field`, in
/// bytes: the kernel writes these in kB, and a bare number is taken as is.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
fn proc_field_bytes(text: &str, field: &str) -> Option<u64> {
    let line = text.lines().find_map(|line| line.strip_prefix(field))?;
    let mut words = line.split_whitespace();
    let amount = words.next()?.parse::<u64>().ok()?;
    match words.next() {
        Some("kB") => amount.checked_mul(1024),
        None => Some(amount),
        Some(_) => None,
    }
}

// Two Win32 calls with no safe wrapper in the dependency tree. The rest of
// the crate keeps `#![deny(unsafe_code)]`.
#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_memory {
    use std::os::windows::io::AsRawHandle;

    use windows_sys::Win32::{
        Foundation::HANDLE,
        System::{
            ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
            SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        },
    };

    /// The working set of a child this process started, through the handle
    /// `Child` owns, which carries every access right the call needs.
    /// `K32GetProcessMemoryInfo` is `GetProcessMemoryInfo` as kernel32
    /// exports it, so no other library is loaded for it.
    pub(super) fn working_set(process: &std::process::Child) -> Option<u64> {
        let size = u32::try_from(size_of::<PROCESS_MEMORY_COUNTERS>()).ok()?;
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: size,
            ..PROCESS_MEMORY_COUNTERS::default()
        };
        // SAFETY: the handle is owned by `process` and open for the whole
        // call; `counters` is a correctly sized PROCESS_MEMORY_COUNTERS with
        // its size given, as the call requires, and lives for the whole
        // call. Failure is reported through the return value, which is
        // checked.
        let succeeded = unsafe {
            K32GetProcessMemoryInfo(process.as_raw_handle() as HANDLE, &mut counters, size)
        };
        if succeeded == 0 {
            return None;
        }
        u64::try_from(counters.WorkingSetSize).ok()
    }

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
        // The guard asks for more than either model must keep, with the
        // worker beside it, and more for the larger model.
        for tier in ModelTier::ALL {
            assert!(min_available_for(tier) > working_set(tier) + WORKER_PEAK);
            assert!(working_set(tier) <= resident_estimate(tier));
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
            assert!(min_available_for(tier) >= working_set(tier) + WORKER_PEAK + HEADROOM);
        }
    }

    /// The estimate: the mapped weights are counted as available already
    /// (on Linux), so only the rest of a loaded model comes back on unload -
    /// never less than nothing.
    #[test]
    fn unloading_gives_back_only_what_is_not_the_mapped_file() {
        assert_eq!(unload_gives_back(F, 0), resident_estimate(F));
        assert_eq!(
            unload_gives_back(A, 5_680_522_464),
            resident_estimate(A) - 5_680_522_464
        );
        assert_eq!(unload_gives_back(F, u64::MAX), 0);
    }

    /// The memory the plan judges by: the machine's own figure with nothing
    /// loaded, plus what the loaded server gives back, less the margin only
    /// while the fast model is the one loaded. The loaded accurate model is
    /// still stood down at the floor itself.
    #[test]
    fn only_a_loaded_fast_model_is_asked_for_the_margin() {
        let released = 3 * GIB;
        assert_eq!(memory_for_plan(5 * GIB, None), 5 * GIB);
        assert_eq!(memory_for_plan(5 * GIB, Some((A, released))), 8 * GIB);
        assert_eq!(
            memory_for_plan(5 * GIB, Some((F, released))),
            8 * GIB - UPGRADE_MARGIN
        );
        assert_eq!(memory_for_plan(0, Some((F, 0))), 0);
        assert_eq!(memory_for_plan(u64::MAX, Some((A, released))), u64::MAX);

        let first = |loaded| {
            let memory = memory_for_plan(MIN_AVAILABLE_FOR_ACCURATE - released, loaded);
            plan(Automatic, BOTH, Some(memory)).map(|plan| plan.first)
        };
        assert_eq!(first(Some((A, released))), Some(A));
        assert_eq!(first(Some((A, released - 1))), Some(F));
        assert_eq!(first(Some((F, released))), Some(F));
        assert_eq!(first(Some((F, released + UPGRADE_MARGIN - 1))), Some(F));
        assert_eq!(first(Some((F, released + UPGRADE_MARGIN))), Some(A));
    }

    /// The Windows probe's passes with 2 GiB held by another process, carried
    /// to a machine with `nothing` MiB available before any model is loaded,
    /// as the report's bands carry them: each load takes from the available
    /// memory what it took on the runner, and the server reports the working
    /// set it reported. Pass 3, with repacked weights
    /// (`bench/reports/windows-memory-2026-11-09`), and the 9B with
    /// `--no-repack` (`runtime-speed.yml` run 43063958237). Credited that
    /// alone, a machine near the floor with the 9B loaded is judged to have
    /// room once the 4B is loaded, and the two swap, document after document
    /// or pause after pause; asked for alpha.16's 1,024 MiB, at every pause
    /// still with `--no-repack`. Asked for the margin, the 4B stays.
    #[test]
    fn a_loaded_fast_model_makes_way_only_with_room_to_spare() {
        // MiB: what the load took from the available memory, and the
        // server's working set then. The 9B right after a document; the 4B
        // right after a document, and after a minute idle.
        const PASS_3: [(u64, u64); 3] = [(7_926, 8_107), (3_929, 4_514), (3_347, 4_445)];
        const NO_REPACK: [(u64, u64); 3] = [(5_631, 5_490), (3_746, 4_514), (3_061, 4_514)];
        const ALPHA_16_MARGIN: u64 = 1_024 * MIB;
        let first = |memory: u64| plan(Automatic, BOTH, Some(memory)).map(|plan| plan.first);
        // A machine with less than the 9B took has nothing left available.
        let available = |nothing: u64, takes: u64| nothing.saturating_sub(takes) * MIB;
        // The machines on which the models swap, credited the working set
        // alone and asked for alpha.16's margin: the first, the last and how
        // many, after a document and after a minute idle. Asked for the
        // margin, none.
        let bands = |[accurate, after_a_document, after_a_minute_idle]: [(u64, u64); 3]| {
            let mut swaps = [[Vec::new(), Vec::new()], [Vec::new(), Vec::new()]];
            for nothing in 6_000..10_000_u64 {
                let (takes, holds) = accurate;
                let with_accurate =
                    memory_for_plan(available(nothing, takes), Some((A, holds * MIB)));
                let stands_down = first(with_accurate) == Some(F);
                assert_eq!(
                    stands_down,
                    (nothing + holds - takes) * MIB < MIN_AVAILABLE_FOR_ACCURATE,
                    "{nothing} MiB"
                );
                for (stage, (takes, holds)) in [after_a_document, after_a_minute_idle]
                    .into_iter()
                    .enumerate()
                {
                    let with_fast = available(nothing, takes);
                    let comes_back = first(memory_for_plan(with_fast, Some((F, holds * MIB))));
                    assert!(
                        !(stands_down && comes_back == Some(A)),
                        "{nothing} MiB: the models swap"
                    );
                    for (margin, found) in [0, ALPHA_16_MARGIN].into_iter().zip(&mut swaps) {
                        let figure = (with_fast + holds * MIB).saturating_sub(margin);
                        if stands_down && first(figure) == Some(A) {
                            found[stage].push(nothing);
                        }
                    }
                }
            }
            swaps.map(|stages| {
                stages.map(|found| (found.first().copied(), found.last().copied(), found.len()))
            })
        };
        assert_eq!(
            bands(PASS_3),
            [
                [
                    (Some(7_607), Some(8_010), 404),
                    (Some(7_094), Some(8_010), 917)
                ],
                [(None, None, 0), (None, None, 0)],
            ]
        );
        assert_eq!(
            bands(NO_REPACK),
            [
                [
                    (Some(7_424), Some(8_332), 909),
                    (Some(6_739), Some(8_332), 1_594)
                ],
                [(None, None, 0), (Some(7_763), Some(8_332), 570)],
            ]
        );
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

    /// Every reading is judged by the gate, so every request asks for the
    /// probabilities it reads; each tier's threshold is its own constant.
    #[test]
    fn token_probabilities_are_asked_for_only_where_something_reads_them() {
        assert_eq!(
            review_below_token_confidence(A),
            Some(REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE)
        );
        assert_eq!(
            review_below_token_confidence(F),
            Some(REVIEW_BELOW_TOKEN_CONFIDENCE_FAST)
        );
        for tier in ModelTier::ALL {
            assert!(wants_token_confidence(tier), "the review gate reads them");
            let threshold = review_below_token_confidence(tier).unwrap();
            assert!((0.0..=1.0).contains(&threshold));
        }
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
        assert_eq!(tier_server_flags(ModelTier::Accurate), ["--no-repack"]);
        assert_eq!(tier_server_flags(ModelTier::Fast), ["--no-repack"]);
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

    #[test]
    fn rss_anon_is_read_from_a_process_status() {
        let status = "Name:\tllama-server\nVmRSS:\t 8949776 kB\nRssAnon:\t 3396644 kB\n\
                      RssFile:\t 5553132 kB\nRssShmem:\t       0 kB\n";
        assert_eq!(rss_anon_from_status(status), Some(3_396_644 * 1024));
        // A process that has exited and not been reaped has no memory map.
        assert_eq!(
            rss_anon_from_status("Name:\tsleep\nState:\tZ (zombie)\n"),
            None
        );
        assert_eq!(rss_anon_from_status("RssAnon:\t 12 MB\n"), None);
        assert_eq!(rss_anon_from_status("RssAnon:\n"), None);
    }

    /// The live reading the runtime credits a loaded model with works on a
    /// real child process here: something, while it runs.
    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn a_running_child_reports_what_its_exit_would_give_back() {
        let mut command = if cfg!(windows) {
            let mut command = std::process::Command::new("ping");
            command.args(["-n", "30", "127.0.0.1"]);
            command
        } else {
            let mut command = std::process::Command::new("sleep");
            command.arg("30");
            command
        };
        let mut child = command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("a child starts");
        let released = memory_released_by(&child);
        let _ = child.kill();
        let _ = child.wait();
        let released = released.expect("readable while it runs");
        assert!(released > 0);
        assert!(released < 1024 * MIB, "{released} bytes for an idle child");
    }
}
