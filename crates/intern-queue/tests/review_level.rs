//! The review level: which finished readings the queue shows as ready and
//! files with no person looking. Strict is what the queue always did;
//! balanced also makes ready a reading whose only doubts are notes
//! (`intern_engine::readiness`). The queue's own holds stand at both, and
//! changing the level judges the waiting documents again from their stored
//! readings. Everything here runs the real journaled file operations on
//! real files; the readings are the engine's own validation of a scripted
//! reply, with the review reasons a test names added to them.

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
    AnalysisTelemetry, DateRole, DigestBudget, DocumentAnalysis, DocumentSource, Evidence,
    ExtractProgress, ModelProposal, PageOrigin, PartyRelation, ProposalStatus, ReviewLevel,
    ReviewReason, SourcePage, distill, engine::finish, fingerprint, validate,
};
use intern_queue::{
    AdmissionEvidence, AdmissionGuard, AdmissionStage,
    pipeline::{
        AnalyzerBoundary, LEARNED_SPELLING_UNCONFIRMED, ModelFailure, NEAR_DUPLICATE, Pipeline,
        PipelineError, PipelineEventSink, PipelineItem, PipelineProgress, ProposalRecord,
        WorkerBoundary, WorkerFailure,
    },
    settings::{AppSettings, SettingsStore},
};
use rusqlite::{Connection, params};
use serde_json::Value;
use tempfile::{TempDir, tempdir};

/// An agreement long enough to fingerprint, signed on the date a document
/// gives.
fn agreement(written: &str) -> String {
    format!(
        "EMPLOYMENT AGREEMENT\n\nThis Employment Agreement is signed {written} by John Smith \
         and Acme Corporation. Acme Corporation employs John Smith as a senior cartographer at \
         its Fictional Harbor office. The position begins on the start date named above and \
         continues until terminated by either party on thirty days written notice. Salary, \
         benefits, and duties are described in the attached schedule, which forms part of this \
         agreement."
    )
}

/// One document: the date it is signed on, and the review reasons its
/// reading carries beyond what validation finds, which is none.
#[derive(Clone)]
struct Doc {
    iso: &'static str,
    written: &'static str,
    notes: Vec<ReviewReason>,
}

fn doc(iso: &'static str, written: &'static str, notes: &[ReviewReason]) -> Doc {
    Doc {
        iso,
        written,
        notes: notes.to_vec(),
    }
}

const APRIL: (&str, &str) = ("2024-04-12", "April 12, 2024");
const MAY: (&str, &str) = ("2024-05-03", "May 3, 2024");
const JUNE: (&str, &str) = ("2024-06-07", "June 7, 2024");
const JULY: (&str, &str) = ("2024-07-05", "July 5, 2024");

/// A document whose reading carries only a note about its type.
fn noted(date: (&'static str, &'static str)) -> Doc {
    doc(date.0, date.1, &[ReviewReason::TypeInferred])
}

/// A document whose reading validation has nothing to say about.
fn plain(date: (&'static str, &'static str)) -> Doc {
    doc(date.0, date.1, &[])
}

fn name_for(iso: &str) -> String {
    format!("{iso} Employment Agreement between John Smith and Acme Corporation.pdf")
}

/// The reader and the model in one: it reads each file as the agreement of
/// the document it was told the file is, and answers for the document it
/// read last, as the queue asks, one document at a time.
#[derive(Default)]
struct Office {
    docs: Mutex<HashMap<String, Doc>>,
    reading: Mutex<Option<Doc>>,
    extractions: AtomicUsize,
    readings: AtomicUsize,
}

impl WorkerBoundary for Office {
    fn extract(
        &self,
        _request_id: &str,
        path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let Some(doc) = self.docs.lock().unwrap().get(&name).cloned() else {
            return Err(WorkerFailure::new("PARSE_FAILED", false, false));
        };
        self.extractions.fetch_add(1, Ordering::SeqCst);
        let source = DocumentSource::from_pages(vec![SourcePage::new(
            1,
            agreement(doc.written),
            PageOrigin::Native,
        )]);
        *self.reading.lock().unwrap() = Some(doc);
        Ok(source)
    }

    fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
        Ok(())
    }

    fn restart(&self) -> Result<(), WorkerFailure> {
        Ok(())
    }
}

impl AnalyzerBoundary for Office {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let doc = self
            .reading
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| ModelFailure::fatal("ANALYSIS_FAILED"))?;
        self.readings.fetch_add(1, Ordering::SeqCst);
        let reply = ModelProposal {
            document_type: Some("Employment Agreement".into()),
            document_date: Some(doc.iso.into()),
            date_role: Some(DateRole::Execution),
            parties: vec!["John Smith".into(), "Acme Corporation".into()],
            party_relation: PartyRelation::Between,
            description: "Employment agreement between John Smith and Acme Corporation covering \
                          duties, salary, and term."
                .into(),
            confidence: 0.94,
            needs_review: false,
            evidence: Evidence {
                date: Some(format!("signed {}", doc.written)),
                document_type: Some("Employment Agreement".into()),
                parties: vec!["by John Smith and Acme Corporation".into()],
            },
            facts: None,
        };
        let digest = distill(source, DigestBudget::default());
        let mut analysis = finish(
            validate(reply, &digest),
            &digest,
            extension,
            existing_names,
            AnalysisTelemetry::default(),
        );
        analysis.text_fingerprint =
            fingerprint::source_fingerprint(source).map(fingerprint::encode);
        assert_eq!(
            (analysis.status, analysis.review_reasons.as_slice()),
            (ProposalStatus::Ready, &[][..]),
            "validation has nothing to say about the agreement"
        );
        // The engine's own word about a reading it has a reason on.
        if !doc.notes.is_empty() {
            analysis.review_reasons.clone_from(&doc.notes);
            analysis.status = ProposalStatus::NeedsReview;
        }
        Ok(analysis)
    }
}

struct Quiet;

impl PipelineEventSink for Quiet {
    fn queue_changed(&self) {}
    fn progress(&self, _progress: PipelineProgress) {}
}

/// What a test does once the queue has stored a document's reading and
/// before it settles it: a Settings save landing in that moment.
type AfterStored = Box<dyn Fn() + Send + Sync>;

/// Admits every file as a local one, and runs [`AfterStored`] when the
/// queue says a document was read to the end.
#[derive(Default)]
struct Admissions {
    after_stored: Mutex<Option<AfterStored>>,
}

impl AdmissionGuard for Admissions {
    fn authorize(
        &self,
        _path: &Path,
        _stage: AdmissionStage,
    ) -> Result<AdmissionEvidence, PipelineError> {
        Ok(AdmissionEvidence::local())
    }

    fn processed(&self, _path: &Path, _hash: &str) {
        if let Some(hook) = self.after_stored.lock().unwrap().as_ref() {
            hook();
        }
    }
}

/// An input folder, an output folder, and a queue over them at a review
/// level, with the real file operations.
struct Rig {
    temp: TempDir,
    inbox: PathBuf,
    filed: PathBuf,
    office: Arc<Office>,
    admissions: Arc<Admissions>,
    pipeline: Pipeline,
}

impl Rig {
    fn new(level: ReviewLevel, automatic_rename: bool) -> Rig {
        let temp = tempdir().unwrap();
        let inbox = temp.path().join("Inbox");
        let filed = temp.path().join("Filed");
        fs::create_dir_all(&inbox).unwrap();
        fs::create_dir_all(&filed).unwrap();
        let inbox = inbox.canonicalize().unwrap();
        let filed = filed.canonicalize().unwrap();
        let office = Arc::new(Office::default());
        let admissions = Arc::new(Admissions::default());
        let store = SettingsStore::new(temp.path().join("settings.json"));
        store
            .save(&AppSettings {
                destination: filed.to_string_lossy().into_owned(),
                automatic_rename,
                review_level: level,
                ..AppSettings::default()
            })
            .unwrap();
        let pipeline = open(temp.path(), &office, &admissions);
        Rig {
            temp,
            inbox,
            filed,
            office,
            admissions,
            pipeline,
        }
    }

    fn settings(&self) -> SettingsStore {
        SettingsStore::new(self.temp.path().join("settings.json"))
    }

    /// The queue opened again over the same data, as after Intern closed.
    fn reopened(&self) -> Pipeline {
        open(self.temp.path(), &self.office, &self.admissions)
    }

    /// Puts `doc` in the input folder as `name` and adds it.
    fn add(&self, name: &str, doc: Doc) -> i64 {
        let path = self.inbox.join(name);
        fs::write(&path, name.as_bytes()).unwrap();
        self.office
            .docs
            .lock()
            .unwrap()
            .insert(name.to_owned(), doc);
        self.pipeline.enqueue_files(&[path]).unwrap().remove(0).id
    }

    /// [`Self::add`], and the queue reads it.
    fn read(&self, name: &str, doc: Doc) -> i64 {
        let id = self.add(name, doc);
        self.pipeline.run_until_idle().unwrap();
        id
    }

    fn item(&self, id: i64) -> PipelineItem {
        item(&self.pipeline, id)
    }

    fn status(&self, id: i64) -> QueueStatus {
        self.item(id).status
    }

    fn record(&self, id: i64) -> ProposalRecord {
        self.item(id).proposal.unwrap()
    }

    /// Settings saved with `change` made, and nothing told to the queue.
    fn save(&self, change: impl FnOnce(&mut AppSettings)) -> AppSettings {
        let store = self.settings();
        let previous = store.load().unwrap();
        let mut next = previous.clone();
        change(&mut next);
        store.save(&next).unwrap();
        previous
    }

    /// The level changed in Settings, as the app saves it: stored first,
    /// then the queue is told what it changed from and to.
    fn set_level(&self, level: ReviewLevel) {
        let previous = self.save(|settings| settings.review_level = level);
        self.pipeline
            .review_level_changed(previous.review_level, level)
            .unwrap();
    }

    fn filed_names(&self) -> Vec<String> {
        let mut names = fs::read_dir(&self.filed)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('.'))
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Rewrites item `id`'s stored record through `edit`.
    fn edit_stored(&self, id: i64, edit: impl FnOnce(&mut Value)) {
        let connection = Connection::open(self.temp.path().join("queue.sqlite3")).unwrap();
        let stored: String = connection
            .query_row(
                "SELECT proposal_json FROM proposals WHERE queue_item_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        let mut record: Value = serde_json::from_str(&stored).unwrap();
        edit(&mut record);
        connection
            .execute(
                "UPDATE proposals SET proposal_json = ?1 WHERE queue_item_id = ?2",
                params![record.to_string(), id],
            )
            .unwrap();
    }
}

fn open(data: &Path, office: &Arc<Office>, admissions: &Arc<Admissions>) -> Pipeline {
    Pipeline::with_local_files(
        data.join("queue.sqlite3"),
        office.clone(),
        office.clone(),
        Arc::new(Quiet),
        SettingsStore::new(data.join("settings.json")),
    )
    .unwrap()
    .with_admission_guard(admissions.clone())
}

fn item(pipeline: &Pipeline, id: i64) -> PipelineItem {
    pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap()
}

const STRICT: ReviewLevel = ReviewLevel::Strict;
const BALANCED: ReviewLevel = ReviewLevel::Balanced;

/// A reading whose only reason is a note about its type waits for a person
/// at strict, as it always did, and is ready at balanced. The record keeps
/// the reason either way, and the reading keeps the engine's own status.
#[test]
fn a_reading_with_only_a_type_note_waits_at_strict_and_is_ready_at_balanced() {
    let strict = Rig::new(STRICT, false);
    let waiting = strict.read("noted.pdf", noted(APRIL));
    assert_eq!(strict.status(waiting), QueueStatus::NeedsReview);
    let record = strict.record(waiting);
    assert_eq!(record.status, ProposalStatus::NeedsReview);
    assert_eq!(record.reasons, ["TYPE_INFERRED"]);

    let balanced = Rig::new(BALANCED, false);
    let ready = balanced.read("noted.pdf", noted(APRIL));
    assert_eq!(balanced.status(ready), QueueStatus::Ready);
    let record = balanced.record(ready);
    assert_eq!(record.status, ProposalStatus::Ready);
    assert_eq!(record.reasons, ["TYPE_INFERRED"], "a note, still said");
    assert_eq!(record.filename, name_for(APRIL.0));
    assert!(!record.approved, "ready is not approved");
    assert_eq!(
        (record.analysis.status, record.analysis.review_reasons),
        (
            ProposalStatus::NeedsReview,
            vec![ReviewReason::TypeInferred]
        ),
        "the reading is stored as the engine made it"
    );
    assert!(
        balanced.filed_names().is_empty(),
        "automatic renaming is off"
    );

    // A reading with no reason at all is ready at both.
    for rig in [&strict, &balanced] {
        let plain = rig.read("plain.pdf", plain(MAY));
        assert_eq!(rig.status(plain), QueueStatus::Ready);
        assert_eq!(rig.record(plain).reasons, Vec::<String>::new());
    }
}

/// With automatic renaming on, the same reading is filed at balanced with
/// no person, and the stored record still says what the engine noted. At
/// strict it waits and nothing is filed.
#[test]
fn at_balanced_automatic_renaming_files_it_and_its_reasons_stay_on_the_record() {
    let balanced = Rig::new(BALANCED, true);
    let filed = balanced.read("noted.pdf", noted(APRIL));
    assert_eq!(balanced.status(filed), QueueStatus::Completed);
    assert_eq!(balanced.filed_names(), [name_for(APRIL.0)]);
    let record = balanced.record(filed);
    assert_eq!(record.reasons, ["TYPE_INFERRED"]);
    assert_eq!(record.analysis.review_reasons, [ReviewReason::TypeInferred]);
    assert!(!record.approved, "filed by itself, approved by nobody");
    assert!(!record.reviewed);

    let strict = Rig::new(STRICT, true);
    let waiting = strict.read("noted.pdf", noted(APRIL));
    assert_eq!(strict.status(waiting), QueueStatus::NeedsReview);
    assert!(strict.filed_names().is_empty());
}

/// A party the document does not settle is no note: it waits at both
/// levels, alone or beside a note, and is never filed by itself.
#[test]
fn a_party_the_document_does_not_settle_waits_at_both_levels() {
    for level in ReviewLevel::ALL {
        let rig = Rig::new(level, true);
        let alone = rig.read(
            "party.pdf",
            doc(APRIL.0, APRIL.1, &[ReviewReason::PartyUnsupported]),
        );
        let beside_a_note = rig.read(
            "both.pdf",
            doc(
                MAY.0,
                MAY.1,
                &[ReviewReason::TypeInferred, ReviewReason::PartyUnsupported],
            ),
        );
        assert_eq!(rig.status(alone), QueueStatus::NeedsReview, "{level:?}");
        assert_eq!(rig.record(alone).reasons, ["PARTY_UNSUPPORTED"]);
        assert_eq!(rig.status(beside_a_note), QueueStatus::NeedsReview);
        assert_eq!(
            rig.record(beside_a_note).reasons,
            ["TYPE_INFERRED", "PARTY_UNSUPPORTED"]
        );
        assert!(rig.filed_names().is_empty(), "{level:?}");
    }
}

/// The queue's own holds stand at balanced. A second scan of a filed
/// document waits as a near duplicate, and a name a learned spelling nobody
/// has checked changed waits for a person, though the reading of each would
/// be ready there.
#[test]
fn a_near_duplicate_and_an_unchecked_spelling_still_hold_at_balanced() {
    let rig = Rig::new(BALANCED, true);
    let first = rig.read("scan-1.pdf", noted(APRIL));
    assert_eq!(rig.status(first), QueueStatus::Completed);
    let rescan = rig.read("scan-2.pdf", noted(APRIL));
    assert_eq!(rig.status(rescan), QueueStatus::NeedsReview);
    let record = rig.record(rescan);
    assert_eq!(record.reasons, ["TYPE_INFERRED", NEAR_DUPLICATE]);
    assert_eq!(
        record.near_duplicate_of.as_deref(),
        Some(name_for(APRIL.0).as_str())
    );
    assert_eq!(rig.filed_names(), [name_for(APRIL.0)], "not filed twice");

    // A spelling learned from one edit and put to use, which nobody has
    // said may file on its own.
    let rig = Rig::new(BALANCED, false);
    let taught = rig.read("taught.pdf", plain(APRIL));
    rig.pipeline
        .approve(
            taught,
            "2024-04-12 Employment Agreement between John Smith and Acme.pdf",
            "A sentence about the agreement.",
        )
        .unwrap();
    let rule = rig.pipeline.learned_rules().unwrap().remove(0);
    assert_eq!(
        (rule.from.as_str(), rule.to.as_str()),
        ("Acme Corporation", "Acme")
    );
    rig.pipeline.use_rule(rule.id).unwrap();
    rig.save(|settings| settings.automatic_rename = true);
    let respelled = rig.read("respelled.pdf", noted(MAY));
    assert_eq!(rig.status(respelled), QueueStatus::NeedsReview);
    let record = rig.record(respelled);
    assert_eq!(
        record.filename,
        "2024-05-03 Employment Agreement between John Smith and Acme.pdf"
    );
    assert_eq!(
        record.reasons,
        ["TYPE_INFERRED", LEARNED_SPELLING_UNCONFIRMED]
    );
    assert_eq!(rig.filed_names().len(), 1, "only the approved one is filed");
}

/// Changing the level judges the waiting documents again from their stored
/// readings, both ways, with no model asked and nothing read again: a
/// noted reading becomes ready at balanced and goes back to review at
/// strict, a reading with a real doubt waits throughout, and one ready at
/// strict is ready throughout. Only readiness moves: the name, the reasons
/// and the revision a reviewer's draft hangs on stay as they were when the
/// document becomes ready.
#[test]
fn changing_the_level_judges_the_waiting_documents_again_both_ways() {
    let rig = Rig::new(STRICT, false);
    let noted_id = rig.read("noted.pdf", noted(APRIL));
    let doubted = rig.read(
        "party.pdf",
        doc(MAY.0, MAY.1, &[ReviewReason::PartyUnsupported]),
    );
    let plain_id = rig.read("plain.pdf", plain(JUNE));
    let statuses = || [noted_id, doubted, plain_id].map(|id| rig.status(id));
    assert_eq!(
        statuses(),
        [
            QueueStatus::NeedsReview,
            QueueStatus::NeedsReview,
            QueueStatus::Ready
        ]
    );
    let as_read = rig.record(noted_id);

    rig.set_level(BALANCED);
    assert_eq!(
        statuses(),
        [
            QueueStatus::Ready,
            QueueStatus::NeedsReview,
            QueueStatus::Ready
        ]
    );
    let made_ready = rig.record(noted_id);
    assert_eq!(made_ready.status, ProposalStatus::Ready);
    assert!(!made_ready.approved);
    assert_eq!(
        ProposalRecord {
            status: ProposalStatus::NeedsReview,
            ..made_ready.clone()
        },
        as_read,
        "nothing but readiness moved"
    );
    assert!(rig.filed_names().is_empty(), "automatic renaming is off");

    rig.set_level(STRICT);
    assert_eq!(
        statuses(),
        [
            QueueStatus::NeedsReview,
            QueueStatus::NeedsReview,
            QueueStatus::Ready
        ]
    );
    let sent_back = rig.record(noted_id);
    assert_eq!(sent_back.status, ProposalStatus::NeedsReview);
    assert_eq!(sent_back.reasons, ["TYPE_INFERRED"]);
    assert_eq!(sent_back.filename, as_read.filename);

    rig.set_level(BALANCED);
    assert_eq!(rig.status(noted_id), QueueStatus::Ready);
    // Saving the same level again changes nothing.
    rig.set_level(BALANCED);
    assert_eq!(
        statuses(),
        [
            QueueStatus::Ready,
            QueueStatus::NeedsReview,
            QueueStatus::Ready
        ]
    );

    assert_eq!(rig.office.readings.load(Ordering::SeqCst), 3, "no model");
    assert_eq!(
        rig.office.extractions.load(Ordering::SeqCst),
        3,
        "no re-read"
    );
}

/// A document the level makes ready follows automatic renaming exactly as a
/// newly ready reading does: filed at once when it is on, and when the
/// queue is paused, left ready until the queue runs again.
#[test]
fn a_document_the_level_makes_ready_follows_automatic_renaming_and_the_pause() {
    let rig = Rig::new(STRICT, true);
    let id = rig.read("noted.pdf", noted(APRIL));
    assert_eq!(rig.status(id), QueueStatus::NeedsReview);
    rig.set_level(BALANCED);
    assert_eq!(rig.status(id), QueueStatus::Completed);
    assert_eq!(rig.filed_names(), [name_for(APRIL.0)]);
    assert_eq!(rig.record(id).reasons, ["TYPE_INFERRED"]);

    let paused = Rig::new(STRICT, true);
    let id = paused.read("noted.pdf", noted(APRIL));
    paused.pipeline.pause();
    paused.set_level(BALANCED);
    assert_eq!(
        paused.status(id),
        QueueStatus::Ready,
        "ready, and not filed"
    );
    assert!(paused.filed_names().is_empty());
    paused.pipeline.resume();
    paused.pipeline.run_until_idle().unwrap();
    assert_eq!(paused.status(id), QueueStatus::Completed);
    assert_eq!(paused.filed_names(), [name_for(APRIL.0)]);
}

/// What a person decided, and what is filed, is no level's to judge again:
/// a name a person edited stays in review as they left it when the level
/// loosens, a name a person approved stays approved and ready when it
/// tightens, and a filed document stays filed either way.
#[test]
fn changing_the_level_leaves_edited_approved_and_filed_records_alone() {
    let rig = Rig::new(STRICT, false);
    let edited = rig.read("edited.pdf", noted(APRIL));
    let filed = rig.read("filed.pdf", noted(MAY));

    // An approval of an edited name whose filing fails: the output folder
    // cannot be made. The document is back in review under the person's
    // name.
    let blocker = rig.temp.path().join("blocker");
    fs::write(&blocker, b"a file where the output folder should be").unwrap();
    let out = rig.filed.to_string_lossy().into_owned();
    rig.save(|settings| {
        settings.destination = blocker.join("Filed").to_string_lossy().into_owned();
    });
    let their_name = "2024-04-12 Offer Letter for John Smith.pdf";
    let refused = rig
        .pipeline
        .approve(edited, their_name, "Their sentence.")
        .unwrap_err();
    assert_eq!(refused.code, "DESTINATION_UNAVAILABLE");
    rig.save(|settings| settings.destination = out);
    let theirs = rig.record(edited);
    assert_eq!(rig.status(edited), QueueStatus::NeedsReview);
    assert!(theirs.reviewed && !theirs.approved);
    assert_eq!(theirs.filename, their_name);

    let filed_name = rig.record(filed).filename;
    rig.pipeline
        .approve(filed, &filed_name, "A sentence about the agreement.")
        .unwrap();
    assert_eq!(rig.status(filed), QueueStatus::Completed);
    let filed_record = rig.record(filed);

    rig.set_level(BALANCED);
    assert_eq!(rig.status(edited), QueueStatus::NeedsReview);
    assert_eq!(rig.record(edited), theirs, "the edited record is as it was");
    assert_eq!(rig.status(filed), QueueStatus::Completed);
    assert_eq!(rig.record(filed), filed_record);
    assert_eq!(rig.filed_names(), [name_for(MAY.0)]);

    // Two readings balanced made ready. A person approves one while the
    // queue is too busy to file it, so the approval waits on the record.
    let approved = rig.read("approved.pdf", noted(JUNE));
    let unapproved = rig.read("unapproved.pdf", noted(JULY));
    assert_eq!(rig.status(approved), QueueStatus::Ready);
    assert_eq!(rig.status(unapproved), QueueStatus::Ready);
    rig.edit_stored(approved, |record| record["approved"] = Value::Bool(true));
    let waiting_approval = rig.record(approved);
    assert!(waiting_approval.approved);

    rig.set_level(STRICT);
    assert_eq!(
        rig.status(unapproved),
        QueueStatus::NeedsReview,
        "the change reached the queue"
    );
    assert_eq!(rig.status(approved), QueueStatus::Ready);
    assert_eq!(rig.record(approved), waiting_approval, "still approved");
    assert_eq!(rig.status(edited), QueueStatus::NeedsReview);
    assert_eq!(rig.record(edited), theirs);
    assert_eq!(rig.status(filed), QueueStatus::Completed);
    assert_eq!(rig.record(filed), filed_record);
    assert_eq!(rig.filed_names(), [name_for(MAY.0)]);

    // The approval is filed when the queue is free, at strict as at any
    // level: the name is the person's, not a reading's.
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.status(approved), QueueStatus::Completed);
    assert_eq!(rig.status(unapproved), QueueStatus::NeedsReview);
    assert_eq!(rig.filed_names(), [name_for(MAY.0), name_for(JUNE.0)]);
}

/// A change back to strict that never reached the queue - Intern closed,
/// or the queue could not be written, after Settings were saved - still
/// files nothing strict holds: the scheduler's pass, the recovery pass and
/// the queue opening each put the ready document back in review, and
/// automatic renaming, on by then, files nothing.
#[test]
fn a_change_back_to_strict_that_never_reached_the_queue_files_nothing() {
    let scenario = |reach: &dyn Fn(&Rig) -> Pipeline| {
        let rig = Rig::new(BALANCED, false);
        let noted_id = rig.read("noted.pdf", noted(APRIL));
        let plain_id = rig.read("plain.pdf", plain(MAY));
        assert_eq!(rig.status(noted_id), QueueStatus::Ready);
        rig.save(|settings| {
            settings.review_level = STRICT;
            settings.automatic_rename = true;
        });
        let pipeline = reach(&rig);
        assert_eq!(item(&pipeline, noted_id).status, QueueStatus::NeedsReview);
        let record = item(&pipeline, noted_id).proposal.unwrap();
        assert_eq!(record.status, ProposalStatus::NeedsReview);
        assert_eq!(record.reasons, ["TYPE_INFERRED"], "nothing added");
        pipeline.run_until_idle().unwrap();
        assert_eq!(item(&pipeline, noted_id).status, QueueStatus::NeedsReview);
        assert_eq!(
            item(&pipeline, plain_id).status,
            QueueStatus::Completed,
            "what strict calls ready is filed"
        );
        assert_eq!(rig.filed_names(), [name_for(MAY.0)]);
    };
    // The scheduler's pass, in the queue that is running.
    scenario(&|rig| {
        rig.pipeline.run_until_idle().unwrap();
        rig.reopened()
    });
    // The recovery pass, which needs no model and runs while paused.
    scenario(&|rig| {
        rig.pipeline.pause();
        rig.pipeline.recover().unwrap();
        assert!(rig.filed_names().is_empty());
        rig.reopened()
    });
    // The queue opening again.
    scenario(&|rig| rig.reopened());
}

/// Settings that cannot be read mean strict: a reading made then waits as
/// strict has it wait, and a ready document only balanced made ready goes
/// back to review at the next recovery pass.
#[test]
fn settings_that_cannot_be_read_mean_strict() {
    let rig = Rig::new(BALANCED, false);
    fs::write(rig.settings().path(), b"{ this is not json").unwrap();
    let during = rig.read("during.pdf", noted(APRIL));
    assert_eq!(rig.status(during), QueueStatus::NeedsReview);
    assert_eq!(rig.record(during).reasons, ["TYPE_INFERRED"]);

    let rig = Rig::new(BALANCED, false);
    let before = rig.read("before.pdf", noted(APRIL));
    assert_eq!(rig.status(before), QueueStatus::Ready);
    fs::write(rig.settings().path(), b"{ this is not json").unwrap();
    rig.pipeline.recover().unwrap();
    assert_eq!(rig.status(before), QueueStatus::NeedsReview);
    assert_eq!(rig.record(before).reasons, ["TYPE_INFERRED"]);
    assert!(rig.filed_names().is_empty());
}

/// Two scans of one document waiting at strict are not both filed when
/// the level makes their readings ready: they are judged one at a time,
/// and the second meets the first one's filing as any later reading would.
#[test]
fn two_scans_the_level_makes_ready_are_not_both_filed() {
    let rig = Rig::new(STRICT, true);
    let first = rig.read("scan-1.pdf", noted(APRIL));
    let second = rig.read("scan-2.pdf", noted(APRIL));
    assert_eq!(rig.status(first), QueueStatus::NeedsReview);
    assert_eq!(rig.status(second), QueueStatus::NeedsReview);
    assert_eq!(rig.record(second).near_duplicate_of, None);

    rig.set_level(BALANCED);

    assert_eq!(rig.status(first), QueueStatus::Completed);
    assert_eq!(rig.status(second), QueueStatus::NeedsReview);
    let held = rig.record(second);
    assert_eq!(held.reasons, ["TYPE_INFERRED", NEAR_DUPLICATE]);
    assert_eq!(
        held.near_duplicate_of.as_deref(),
        Some(name_for(APRIL.0).as_str())
    );
    assert_eq!(rig.filed_names(), [name_for(APRIL.0)]);

    // Held as a near duplicate, it is no level's to make ready.
    rig.set_level(STRICT);
    rig.set_level(BALANCED);
    assert_eq!(rig.status(second), QueueStatus::NeedsReview);
    assert_eq!(rig.record(second), held);
}

/// A document held as a near duplicate when it was read stays held when the
/// level changes, also once the filing it repeats is gone: the hold is the
/// queue's own, and no level lifts it. The rename taken back is a person's
/// decision, and stays in review too.
#[test]
fn a_document_held_as_a_near_duplicate_is_no_levels_to_make_ready() {
    let rig = Rig::new(STRICT, true);
    let original = rig.read("scan-1.pdf", plain(APRIL));
    assert_eq!(rig.status(original), QueueStatus::Completed);
    let rescan = rig.read("scan-2.pdf", noted(APRIL));
    let held = rig.record(rescan);
    assert_eq!(rig.status(rescan), QueueStatus::NeedsReview);
    assert_eq!(held.reasons, ["TYPE_INFERRED", NEAR_DUPLICATE]);

    rig.pipeline.undo(original).unwrap();
    assert_eq!(rig.status(original), QueueStatus::NeedsReview);
    assert!(rig.filed_names().is_empty());

    rig.set_level(BALANCED);
    assert_eq!(rig.status(rescan), QueueStatus::NeedsReview);
    assert_eq!(rig.record(rescan), held);
    assert_eq!(rig.status(original), QueueStatus::NeedsReview);
    assert!(rig.filed_names().is_empty(), "nothing is filed again");
}

/// A document left in review for something no level decides stays there:
/// one once held for a learned spelling that was forgotten since, whose
/// reading both levels call ready. The level changing is not what would
/// make it ready, so the level changing does not.
#[test]
fn a_document_both_levels_judge_alike_is_left_where_it_is() {
    let rig = Rig::new(STRICT, false);
    let taught = rig.read("taught.pdf", plain(APRIL));
    rig.pipeline
        .approve(
            taught,
            "2024-04-12 Employment Agreement between John Smith and Acme.pdf",
            "A sentence about the agreement.",
        )
        .unwrap();
    let rule = rig.pipeline.learned_rules().unwrap().remove(0);
    rig.pipeline.use_rule(rule.id).unwrap();
    let held = rig.read("held.pdf", plain(MAY));
    assert_eq!(rig.status(held), QueueStatus::NeedsReview);
    assert_eq!(rig.record(held).reasons, [LEARNED_SPELLING_UNCONFIRMED]);
    rig.pipeline.forget_rule(rule.id).unwrap();
    let released = rig.record(held);
    assert_eq!(rig.status(held), QueueStatus::NeedsReview);
    assert_eq!(released.reasons, Vec::<String>::new());
    assert_eq!(released.filename, name_for(MAY.0));

    rig.set_level(BALANCED);
    assert_eq!(rig.status(held), QueueStatus::NeedsReview);
    assert_eq!(rig.record(held), released);
    rig.set_level(STRICT);
    assert_eq!(rig.status(held), QueueStatus::NeedsReview);
    assert_eq!(rig.record(held), released);
}

/// A level saved while a document is being read finds it not yet waiting,
/// so the save's judging again passes it by. The queue asks the level once
/// more as the reading settles: a reading judged at strict becomes ready
/// and is filed when the level is balanced by then, and one judged at
/// balanced goes to review, unfiled, when it is strict by then.
#[test]
fn a_level_saved_while_a_document_is_read_judges_it_once_it_waits() {
    for (read_at, saved_meanwhile, settles) in [
        (STRICT, BALANCED, QueueStatus::Completed),
        (BALANCED, STRICT, QueueStatus::NeedsReview),
    ] {
        let rig = Rig::new(read_at, true);
        let store = rig.settings();
        *rig.admissions.after_stored.lock().unwrap() = Some(Box::new(move || {
            let mut settings = store.load().unwrap();
            settings.review_level = saved_meanwhile;
            store.save(&settings).unwrap();
        }));
        let id = rig.read("noted.pdf", noted(APRIL));
        assert_eq!(rig.status(id), settles, "read at {read_at:?}");
        assert_eq!(rig.record(id).reasons, ["TYPE_INFERRED"]);
        assert_eq!(
            rig.filed_names().len(),
            usize::from(settles == QueueStatus::Completed)
        );
    }
}
