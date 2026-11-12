//! Documents waiting on a person when a new release is installed are
//! validated again under its rules from the model's stored reply, with no
//! model call - when the request a fresh reading would send is the one that
//! reply answered, to the same model file, and nobody has acted on them.
//! Everything here runs the real engine's evidence pipeline, the real
//! journaled file operations, and real files.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use intern_core::QueueStatus;
use intern_engine::{
    DateRole, DocumentAnalysis, DocumentSource, Engine, EngineResult, ExtractProgress, ModelFacts,
    ModelProposal, ModelRequest, NoModel, PageOrigin, PartyFact, PartyRole, Proposer,
    ProposerReply, SourcePage, TokenConfidence,
};
use intern_queue::{
    AdmissionEvidence, AdmissionGuard, AdmissionStage,
    pipeline::{
        AnalyzerBoundary, ModelFailure, Pipeline, PipelineError, PipelineEventSink, PipelineItem,
        PipelineProgress, Revalidation, RevalidationOutlook, WorkerBoundary, WorkerFailure,
    },
    settings::{AppSettings, SettingsStore},
};
use rusqlite::{Connection, params};
use serde_json::Value;
use tempfile::{TempDir, tempdir};

const AGREEMENT: &str = "EMPLOYMENT AGREEMENT\n\n\
    This Employment Agreement is signed April 12, 2024 by John Smith and Acme Corporation.\n\n\
    John Smith will serve Acme Corporation as its controller from May 1, 2024.";

/// How sure the model is of its date and party tokens: enough for no gate,
/// below the one a later release adds.
const UNSURE: TokenConfidence = TokenConfidence {
    min: 0.6,
    mean: 0.9,
    tokens: 14,
};

/// The review gate on token confidence that the later release brings.
const STRICTER: f32 = 0.8;

/// A model reading an employment agreement from the evidence lines its
/// request shows: the type, the signing date and both parties, each cited
/// to the first line that states it, and as sure of its tokens as
/// [`UNSURE`]. It runs behind the local server's context, as the app's does.
struct AgreementReader;

impl Proposer for AgreementReader {
    fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        let handles = request.evidence.as_ref().expect("an evidence request");
        let cite = |needle: &str| -> Vec<String> {
            request
                .prompt
                .lines()
                .filter_map(|line| line.strip_prefix('['))
                .filter_map(|line| line.split_once("] "))
                .filter(|(_, text)| text.contains(needle))
                .filter_map(|(handle, _)| handles.resolve(handle))
                .map(str::to_owned)
                .take(1)
                .collect()
        };
        let party = |name: &str, role| PartyFact {
            name: name.into(),
            role: Some(role),
            evidence: cite(name),
        };
        Ok(ModelProposal::from_facts(
            ModelFacts {
                document_type: Some("Employment Agreement".into()),
                type_evidence: cite("EMPLOYMENT AGREEMENT"),
                document_date: Some("2024-04-12".into()),
                date_role: Some(DateRole::Execution),
                date_evidence: cite("April 12, 2024"),
                parties: vec![
                    party("John Smith", PartyRole::Employee),
                    party("Acme Corporation", PartyRole::Employer),
                ],
                ..ModelFacts::default()
            },
            0.92,
            false,
        ))
    }

    fn propose_measured(&self, request: &ModelRequest) -> EngineResult<ProposerReply> {
        Ok(ProposerReply {
            proposal: self.propose(request)?,
            token_confidence: Some(UNSURE),
            token_trace: None,
            timings: None,
        })
    }

    fn context_tokens(&self) -> Option<usize> {
        NoModel::local().context_tokens()
    }
}

/// The local runtime as the queue meets it across an update: it reads with
/// the real engine under the review gate the running release sets, and
/// records the model file it pins; asked to validate a stored reading
/// again, it does so with the real engine over no model at all, under the
/// gate as it is then, for the model file it pins then.
struct EvidenceRuntime {
    gate: Mutex<Option<f32>>,
    pin: Mutex<String>,
    /// The runtime cannot tell yet which model would read a document.
    unsure: Mutex<bool>,
    readings: AtomicUsize,
    revalidations: AtomicUsize,
}

impl EvidenceRuntime {
    fn new(gate: Option<f32>) -> Self {
        Self {
            gate: Mutex::new(gate),
            pin: Mutex::new("9b-file-sha256".into()),
            unsure: Mutex::new(false),
            readings: AtomicUsize::new(0),
            revalidations: AtomicUsize::new(0),
        }
    }

    fn engine(&self, model: Box<dyn Proposer>) -> Engine {
        let engine = Engine::with_proposer(model);
        match *self.gate.lock().unwrap() {
            Some(gate) => engine.with_min_token_confidence(gate),
            None => engine,
        }
    }
}

impl AnalyzerBoundary for EvidenceRuntime {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        self.readings.fetch_add(1, Ordering::SeqCst);
        let mut analysis = self
            .engine(Box::new(AgreementReader))
            .analyze(source, extension, existing_names)
            .map_err(|error| ModelFailure::fatal(error.code().as_str()))?;
        analysis.telemetry.model_sha256 = Some(self.pin.lock().unwrap().clone());
        Ok(analysis)
    }

    /// The model file is known without the text: a reading by another one
    /// is declined before the document is read for it.
    fn may_revalidate(&self, stored: &DocumentAnalysis) -> RevalidationOutlook {
        if *self.unsure.lock().unwrap() {
            RevalidationOutlook::NotNow
        } else if stored.telemetry.model_sha256.as_deref()
            == Some(self.pin.lock().unwrap().as_str())
        {
            RevalidationOutlook::Possible
        } else {
            RevalidationOutlook::Declined
        }
    }

    fn revalidate(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
        stored: &DocumentAnalysis,
    ) -> Revalidation {
        self.revalidations.fetch_add(1, Ordering::SeqCst);
        if stored.telemetry.model_sha256.as_deref() != Some(self.pin.lock().unwrap().as_str()) {
            return Revalidation::Declined;
        }
        match self.engine(Box::new(NoModel::local())).revalidate(
            source,
            extension,
            existing_names,
            stored,
        ) {
            Some(mut analysis) => {
                analysis
                    .telemetry
                    .model_sha256
                    .clone_from(&stored.telemetry.model_sha256);
                Revalidation::Revalidated(Box::new(analysis))
            }
            None => Revalidation::Declined,
        }
    }
}

/// What happens while a request is being read, as a person acting on the
/// window meanwhile.
type DuringRead = Box<dyn Fn(&str) + Send + Sync>;

/// A reader that reads each file as the text it was told the file holds,
/// and remembers every request it was sent and every one it was told to
/// stop.
#[derive(Default)]
struct TextWorker {
    texts: Mutex<HashMap<String, String>>,
    requests: Mutex<Vec<String>>,
    cancels: Mutex<Vec<String>>,
    during_read: Mutex<Option<DuringRead>>,
}

impl TextWorker {
    fn reads(&self, name: &str, text: &str) {
        self.texts
            .lock()
            .unwrap()
            .insert(name.to_owned(), text.to_owned());
    }

    fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }

    /// How many requests to read item `id`'s document for validation the
    /// reader was sent.
    fn revalidation_reads(&self) -> usize {
        self.requests()
            .iter()
            .filter(|request| request.starts_with("revalidate-"))
            .count()
    }
}

impl WorkerBoundary for TextWorker {
    fn extract(
        &self,
        request_id: &str,
        path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        self.requests.lock().unwrap().push(request_id.to_owned());
        if let Some(during) = self.during_read.lock().unwrap().as_ref() {
            during(request_id);
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        self.texts
            .lock()
            .unwrap()
            .get(&name)
            .map(|text| {
                DocumentSource::from_pages(vec![SourcePage::new(1, text, PageOrigin::Native)])
            })
            .ok_or_else(|| WorkerFailure::new("PARSE_FAILED", false, false))
    }

    fn cancel(&self, request_id: &str) -> Result<(), WorkerFailure> {
        self.cancels.lock().unwrap().push(request_id.to_owned());
        Ok(())
    }

    fn restart(&self) -> Result<(), WorkerFailure> {
        Ok(())
    }
}

/// Admits every file as a local one, and remembers each document the
/// pipeline said it read to the end.
#[derive(Default)]
struct Admissions {
    processed: Mutex<Vec<PathBuf>>,
}

impl AdmissionGuard for Admissions {
    fn authorize(
        &self,
        _path: &Path,
        _stage: AdmissionStage,
    ) -> Result<AdmissionEvidence, PipelineError> {
        Ok(AdmissionEvidence::local())
    }

    fn processed(&self, path: &Path, _hash: &str) {
        self.processed.lock().unwrap().push(path.to_path_buf());
    }
}

struct Quiet;

impl PipelineEventSink for Quiet {
    fn queue_changed(&self) {}
    fn progress(&self, _progress: PipelineProgress) {}
}

/// An inbox, an output folder, a queue database, and the reader and model
/// every release of the app shares - each `release` a launch of the app at
/// that version over the same data.
struct Rig {
    temp: TempDir,
    inbox: PathBuf,
    filed: PathBuf,
    worker: Arc<TextWorker>,
    runtime: Arc<EvidenceRuntime>,
    admissions: Arc<Admissions>,
}

impl Rig {
    fn new(gate: Option<f32>, automatic_rename: bool) -> Rig {
        let temp = tempdir().unwrap();
        let inbox = temp.path().join("Inbox");
        let filed = temp.path().join("Filed");
        fs::create_dir_all(&inbox).unwrap();
        fs::create_dir_all(&filed).unwrap();
        let rig = Rig {
            inbox: inbox.canonicalize().unwrap(),
            filed: filed.canonicalize().unwrap(),
            temp,
            worker: Arc::new(TextWorker::default()),
            runtime: Arc::new(EvidenceRuntime::new(gate)),
            admissions: Arc::new(Admissions::default()),
        };
        rig.settings()
            .save(&AppSettings {
                destination: rig.filed.to_string_lossy().into_owned(),
                automatic_rename,
                ..AppSettings::default()
            })
            .unwrap();
        rig
    }

    fn settings(&self) -> SettingsStore {
        SettingsStore::new(self.temp.path().join("settings.json"))
    }

    fn database(&self) -> PathBuf {
        self.temp.path().join("queue.sqlite3")
    }

    /// The app at version `release`, launched over this rig's data.
    fn release(&self, release: &str) -> Pipeline {
        Pipeline::with_local_files(
            self.database(),
            self.worker.clone(),
            self.runtime.clone(),
            Arc::new(Quiet),
            self.settings(),
        )
        .unwrap()
        .with_admission_guard(self.admissions.clone())
        .with_release(release)
    }

    /// Puts an agreement named `name` in the inbox and adds it.
    fn add(&self, pipeline: &Pipeline, name: &str) -> i64 {
        let path = self.inbox.join(name);
        fs::write(&path, format!("{name}: {AGREEMENT}")).unwrap();
        self.worker.reads(name, AGREEMENT);
        pipeline.enqueue_files(&[path]).unwrap().remove(0).id
    }

    /// The record stored for item `id`, exactly as it is written.
    fn stored(&self, id: i64) -> String {
        Connection::open(self.database())
            .unwrap()
            .query_row(
                "SELECT proposal_json FROM proposals WHERE queue_item_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap()
    }

    /// Rewrites item `id`'s stored record through `edit`.
    fn edit_stored(&self, id: i64, edit: impl FnOnce(&mut Value)) {
        let mut record: Value = serde_json::from_str(&self.stored(id)).unwrap();
        edit(&mut record);
        Connection::open(self.database())
            .unwrap()
            .execute(
                "UPDATE proposals SET proposal_json = ?1 WHERE queue_item_id = ?2",
                params![record.to_string(), id],
            )
            .unwrap();
    }

    fn revalidated(&self, id: i64) -> bool {
        self.worker.requests().contains(&format!("revalidate-{id}"))
    }
}

fn item(pipeline: &Pipeline, id: i64) -> PipelineItem {
    pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap()
}

fn reasons(pipeline: &Pipeline, id: i64) -> Vec<String> {
    item(pipeline, id).proposal.unwrap().reasons
}

/// A document read under one release waits in review or ready; the next
/// release adds a review gate. Launched, it reads what has arrived first,
/// then validates the waiting document again from its stored reply: the
/// gate sends it to review under the same name, and no model is asked. Each
/// document is validated once per release, and never by the release that
/// read it.
#[test]
fn a_waiting_reading_is_validated_again_under_a_new_release_without_the_model() {
    let rig = Rig::new(None, false);
    let first = rig.release("0.1.0-alpha.18");
    let waiting = rig.add(&first, "waiting.pdf");
    first.run_until_idle().unwrap();
    let read = item(&first, waiting);
    assert_eq!(read.status, QueueStatus::Ready, "{:?}", read.proposal);
    let read = read.proposal.unwrap();
    assert_eq!(read.revision, 1);
    assert!(read.analysis.telemetry.request_sha256.is_some());
    // The release that read it has nothing to validate again.
    assert_eq!(first.revalidate_waiting().unwrap(), 0);
    assert_eq!(rig.runtime.revalidations.load(Ordering::SeqCst), 0);
    drop(first);

    *rig.runtime.gate.lock().unwrap() = Some(STRICTER);
    let next = rig.release("0.1.0-alpha.19");
    let arrived = rig.add(&next, "arrived.pdf");
    next.run_until_idle().unwrap();

    assert_eq!(
        rig.worker.requests(),
        vec![
            format!("queue-{waiting}-1"),
            format!("queue-{arrived}-1"),
            format!("revalidate-{waiting}"),
        ],
        "what arrived is read first"
    );
    assert_eq!(rig.runtime.readings.load(Ordering::SeqCst), 2, "no model");
    assert_eq!(rig.runtime.revalidations.load(Ordering::SeqCst), 1);
    let again = item(&next, waiting);
    assert_eq!(again.status, QueueStatus::NeedsReview);
    let again = again.proposal.unwrap();
    assert_eq!(again.revision, read.revision, "only its readiness moved");
    assert_eq!(again.filename, read.filename);
    assert!(
        again
            .reasons
            .iter()
            .any(|reason| reason == "LOW_CONFIDENCE"),
        "{:?}",
        again.reasons
    );
    assert_eq!(again.analysis.model_proposal, read.analysis.model_proposal);
    // The intake hears the document was read through, as after a reading.
    let path = rig.inbox.join("waiting.pdf");
    let processed = rig.admissions.processed.lock().unwrap().clone();
    assert_eq!(processed.iter().filter(|done| **done == path).count(), 2);

    // Once per release.
    assert_eq!(next.revalidate_waiting().unwrap(), 0);
    next.run_until_idle().unwrap();
    assert_eq!(rig.runtime.revalidations.load(Ordering::SeqCst), 1);
}

/// The window keeps a reviewer's draft of a name and its sentence for as
/// long as the proposal's revision stays, as for a Settings recompose. A
/// document validated again whose readiness and reasons alone changed keeps
/// its revision, so what the reviewer typed into it survives; one whose name
/// or sentence the new release composes differently moves it, and the
/// window shows the new proposal.
#[test]
fn only_a_new_name_or_sentence_moves_the_revision_of_a_reading_validated_again() {
    let rig = Rig::new(None, false);
    let first = rig.release("0.1.0-alpha.18");
    let ids = ["gated.pdf", "renamed.pdf", "redescribed.pdf"].map(|name| rig.add(&first, name));
    let [_, renamed, redescribed] = ids;
    first.run_until_idle().unwrap();
    let read = ids.map(|id| item(&first, id).proposal.unwrap());
    drop(first);
    // As an earlier release composed them.
    rig.edit_stored(renamed, |record| {
        record["filename"] = Value::from("2024-04-12 Agreement.pdf");
    });
    rig.edit_stored(redescribed, |record| {
        record["description"] = Value::from("An agreement.");
    });

    *rig.runtime.gate.lock().unwrap() = Some(STRICTER);
    let next = rig.release("0.1.0-alpha.19");
    assert_eq!(next.revalidate_waiting().unwrap(), 3);

    let again = ids.map(|id| item(&next, id));
    for (validated, before) in again.iter().zip(&read) {
        assert_eq!(validated.status, QueueStatus::NeedsReview);
        let record = validated.proposal.as_ref().unwrap();
        assert!(
            record
                .reasons
                .iter()
                .any(|reason| reason == "LOW_CONFIDENCE"),
            "{:?}",
            record.reasons
        );
        assert_eq!(record.filename, before.filename);
        assert_eq!(record.description, before.description);
    }
    let revisions = again.map(|validated| validated.proposal.unwrap().revision);
    assert_eq!(
        revisions,
        [read[0].revision, read[1].revision + 1, read[2].revision + 1],
        "only a new name or sentence moves it"
    );
    assert_eq!(read.map(|record| record.revision), [1, 1, 1]);
    assert_eq!(rig.runtime.readings.load(Ordering::SeqCst), 3, "no model");
}

/// A stored reply stands in for a fresh reading only for the same request
/// to the same model file. A document the reader now reads differently, one
/// read before readings recorded their request, and - after another update
/// that pins a new model file - every document, are left exactly as they
/// were. The one read before requests were recorded is not even read
/// again, and neither is any document once the model file has changed: the
/// runtime says so from the stored reading alone, before any OCR.
#[test]
fn a_reading_whose_request_or_model_would_differ_is_left_as_it_was() {
    let rig = Rig::new(None, false);
    let first = rig.release("0.1.0-alpha.18");
    let reread = rig.add(&first, "reread.pdf");
    let unrecorded = rig.add(&first, "unrecorded.pdf");
    let control = rig.add(&first, "control.pdf");
    first.run_until_idle().unwrap();
    drop(first);
    rig.edit_stored(unrecorded, |record| {
        record["analysis"]["telemetry"]
            .as_object_mut()
            .unwrap()
            .remove("requestSha256");
    });
    rig.worker
        .reads("reread.pdf", &format!("{AGREEMENT}\n\nPage 1 of 1"));
    let before = [reread, unrecorded].map(|id| rig.stored(id));

    *rig.runtime.gate.lock().unwrap() = Some(STRICTER);
    let next = rig.release("0.1.0-alpha.19");
    assert_eq!(next.revalidate_waiting().unwrap(), 1);

    assert_eq!(item(&next, control).status, QueueStatus::NeedsReview);
    for (id, stored) in [reread, unrecorded].into_iter().zip(&before) {
        assert_eq!(item(&next, id).status, QueueStatus::Ready);
        assert_eq!(&rig.stored(id), stored, "item {id} was left as it was");
    }
    assert!(rig.revalidated(reread) && rig.revalidated(control));
    assert!(!rig.revalidated(unrecorded), "nothing to compare it with");
    drop(next);

    // Another update, with another model file pinned: the gate is lifted
    // again, which would make every document ready - but no stored reply
    // is the new model's.
    *rig.runtime.gate.lock().unwrap() = None;
    *rig.runtime.pin.lock().unwrap() = "another-file-sha256".into();
    let after = [reread, unrecorded, control].map(|id| rig.stored(id));
    let reads = rig.worker.revalidation_reads();
    let last = rig.release("0.1.0-alpha.20");
    assert_eq!(last.revalidate_waiting().unwrap(), 0);
    for (id, stored) in [reread, unrecorded, control].into_iter().zip(&after) {
        assert_eq!(&rig.stored(id), stored, "item {id} was left as it was");
    }
    assert_eq!(item(&last, control).status, QueueStatus::NeedsReview);
    assert_eq!(rig.worker.revalidation_reads(), reads, "nothing read again");
}

/// A runtime that cannot tell yet which model would read a document - none
/// checked since the update - has nothing read for it, and the documents
/// stay due: validated once it can tell.
#[test]
fn a_runtime_that_cannot_tell_yet_has_nothing_read_and_is_asked_again() {
    let rig = Rig::new(None, false);
    let first = rig.release("0.1.0-alpha.18");
    let id = rig.add(&first, "waiting.pdf");
    first.run_until_idle().unwrap();
    drop(first);

    *rig.runtime.gate.lock().unwrap() = Some(STRICTER);
    *rig.runtime.unsure.lock().unwrap() = true;
    let next = rig.release("0.1.0-alpha.19");
    assert_eq!(next.revalidate_waiting().unwrap(), 0);
    next.run_until_idle().unwrap();
    assert_eq!(rig.worker.revalidation_reads(), 0, "nothing read");
    assert_eq!(item(&next, id).status, QueueStatus::Ready);

    *rig.runtime.unsure.lock().unwrap() = false;
    assert_eq!(next.revalidate_waiting().unwrap(), 1);
    assert_eq!(item(&next, id).status, QueueStatus::NeedsReview);
    assert_eq!(rig.runtime.readings.load(Ordering::SeqCst), 1, "no model");
}

/// A person who cancels or removes a waiting document while it is being
/// read for validation stops that reading, and nothing it made of the
/// document is kept: the canceled document, retried, is read again from
/// the start - even when the reading finished as the cancel landed.
#[test]
fn a_document_canceled_or_removed_while_read_for_validation_keeps_nothing() {
    let rig = Rig::new(None, false);
    let first = rig.release("0.1.0-alpha.18");
    let canceled = rig.add(&first, "canceled.pdf");
    let removed = rig.add(&first, "removed.pdf");
    first.run_until_idle().unwrap();
    drop(first);
    let before = rig.stored(canceled);

    *rig.runtime.gate.lock().unwrap() = Some(STRICTER);
    let next = Arc::new(rig.release("0.1.0-alpha.19"));
    let window = Arc::downgrade(&next);
    *rig.worker.during_read.lock().unwrap() = Some(Box::new(move |request: &str| {
        let pipeline = window.upgrade().unwrap();
        if request == format!("revalidate-{canceled}") {
            pipeline.cancel(canceled).unwrap();
        } else if request == format!("revalidate-{removed}") {
            pipeline.remove(removed, false).unwrap();
        }
    }));
    assert_eq!(next.revalidate_waiting().unwrap(), 0);
    *rig.worker.during_read.lock().unwrap() = None;

    assert_eq!(
        rig.worker.cancels.lock().unwrap().clone(),
        vec![
            format!("revalidate-{canceled}"),
            format!("revalidate-{removed}")
        ],
        "each reading was stopped"
    );
    assert_eq!(item(&next, canceled).status, QueueStatus::Canceled);
    assert_eq!(rig.stored(canceled), before, "left as it was");
    assert!(next.list().unwrap().iter().all(|item| item.id != removed));

    let reads = rig.worker.requests().len();
    next.retry(canceled).unwrap();
    next.run_until_idle().unwrap();
    assert_eq!(
        rig.worker.requests()[reads..],
        [format!("queue-{canceled}-1")],
        "read again from the start"
    );
    assert_eq!(item(&next, canceled).status, QueueStatus::NeedsReview);
}

/// Nothing a person has acted on is validated again: an approval waiting for
/// the queue, a name approved whose filing failed, a rename taken back, a
/// document filed or kept. Each stays exactly as it was - not read again -
/// while the untouched document beside them is validated.
#[test]
fn a_reading_someone_acted_on_is_never_validated_again() {
    let rig = Rig::new(None, false);
    let first = rig.release("0.1.0-alpha.18");
    let names = [
        "untouched.pdf",
        "waiting.pdf",
        "unfiled.pdf",
        "undone.pdf",
        "filed.pdf",
        "kept.pdf",
    ];
    let ids = names.map(|name| rig.add(&first, name));
    let [untouched, waiting, unfiled, undone, filed, kept] = ids;
    first.run_until_idle().unwrap();
    for id in ids {
        assert_eq!(item(&first, id).status, QueueStatus::Ready);
    }
    let name_of = |id| item(&first, id).proposal.unwrap().filename;

    // Approved while the queue was busy: the approval waits on the record.
    rig.edit_stored(waiting, |record| record["approved"] = Value::Bool(true));
    // Approved with an edit, but the output folder cannot be made: the
    // filing fails, and the document is back in review, the person's.
    let blocked = rig.temp.path().join("blocked");
    fs::write(&blocked, b"a file, not a folder").unwrap();
    let mut settings = rig.settings().load().unwrap();
    let destination = settings.destination.clone();
    settings.destination = blocked.join("Filed").to_string_lossy().into_owned();
    rig.settings().save(&settings).unwrap();
    let edited = "2024-04-12 Employment Agreement - by hand.pdf";
    let _ = first.approve(unfiled, edited, "Edited by hand.");
    settings.destination = destination;
    rig.settings().save(&settings).unwrap();
    assert_eq!(item(&first, unfiled).status, QueueStatus::NeedsReview);
    first
        .approve(undone, &name_of(undone), "An employment agreement.")
        .unwrap();
    first.undo(undone).unwrap();
    assert_eq!(item(&first, undone).status, QueueStatus::NeedsReview);
    first
        .approve(filed, &name_of(filed), "An employment agreement.")
        .unwrap();
    first.keep_original(kept).unwrap();
    drop(first);
    let acted_on = [waiting, unfiled, undone, filed, kept];
    let before = acted_on.map(|id| rig.stored(id));

    *rig.runtime.gate.lock().unwrap() = Some(STRICTER);
    let next = rig.release("0.1.0-alpha.19");
    assert_eq!(next.revalidate_waiting().unwrap(), 1);

    assert_eq!(item(&next, untouched).status, QueueStatus::NeedsReview);
    for (id, stored) in acted_on.into_iter().zip(&before) {
        assert_eq!(&rig.stored(id), stored, "item {id} was left as it was");
        assert!(!rig.revalidated(id), "item {id} was not read again");
    }
    assert_eq!(item(&next, waiting).status, QueueStatus::Ready);
    assert_eq!(item(&next, unfiled).proposal.unwrap().filename, edited);
    assert!(
        reasons(&next, undone)
            .iter()
            .any(|reason| reason == "UNDONE")
    );
    assert_eq!(item(&next, filed).status, QueueStatus::Completed);
    assert_eq!(item(&next, kept).status, QueueStatus::Completed);
}

/// A document the new rules make ready is treated as any reading that came
/// out ready: filed at once when renaming is automatic, and left ready for a
/// person when it is not.
#[test]
fn a_reading_that_becomes_ready_follows_the_automatic_rename_setting() {
    for automatic in [false, true] {
        let rig = Rig::new(Some(STRICTER), automatic);
        let first = rig.release("0.1.0-alpha.18");
        let id = rig.add(&first, "held.pdf");
        first.run_until_idle().unwrap();
        assert_eq!(item(&first, id).status, QueueStatus::NeedsReview);
        assert!(
            reasons(&first, id)
                .iter()
                .any(|reason| reason == "LOW_CONFIDENCE")
        );
        drop(first);

        *rig.runtime.gate.lock().unwrap() = None;
        let next = rig.release("0.1.0-alpha.19");
        assert_eq!(next.revalidate_waiting().unwrap(), 1);

        let again = item(&next, id);
        let name = again.proposal.as_ref().unwrap().filename.clone();
        assert!(again.proposal.unwrap().reasons.is_empty());
        if automatic {
            assert_eq!(again.status, QueueStatus::Completed);
            assert!(rig.filed.join(&name).exists(), "filed as {name}");
            assert!(!rig.inbox.join("held.pdf").exists());
        } else {
            assert_eq!(again.status, QueueStatus::Ready);
            assert!(rig.inbox.join("held.pdf").exists(), "waits for a person");
        }
        assert_eq!(rig.runtime.readings.load(Ordering::SeqCst), 1);
    }
}
