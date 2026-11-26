//! Which finished readings count as ready.
//!
//! The engine reports every doubt it has about a reading as a review reason,
//! and its own status is ready only when it raised none
//! ([`DocumentAnalysis::status`]). That does not change with the level: a
//! recording, a stored reading and the quick-then-accurate cascade
//! ([`crate::tiers::escalation_with`]) all go on reading the same status and
//! the same reasons.
//!
//! A review level is applied on top. It says which readings are shown as
//! ready and may be filed with no person looking. [`readiness`] is the only
//! place the rule is written: the queue asks it for every reading it stores,
//! and the benchmark asks it for every reading it scores.
//!
//! A few reasons are minor at every level, strict included ([`MINOR`]): a
//! missing date the queue has filled in from the file itself, a type the
//! document's own title or layout gave, and a description that is not a
//! usable sentence or says more than the document does. None of them
//! touches a fact the name is made of.
//!
//! A level never takes a reason off a reading. A ready reading can carry
//! reasons: there they are notes for whoever looks, not holds.
//!
//! [`name_confidence`] is the figure a person is shown beside a reading's
//! name. It is written here, beside the rule, so that the two cannot
//! disagree: a reading in review never shows as surer than a ready one.

use serde::{Deserialize, Deserializer, Serialize};

use crate::domain::{DocumentAnalysis, ProposalStatus, ReviewReason};
use crate::naming::{DEFAULT_TYPE, Spelling, type_segment};
use crate::validate::READY_CONFIDENCE;

/// How much doubt holds a reading for a person.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewLevel {
    /// Ready only when the engine raised no review reason but minor ones
    /// ([`MINOR`]). The default.
    #[default]
    Strict,
    /// Also ready when the only doubts are about how the type or the
    /// description is worded, or about the model's token probabilities, and
    /// the reading has a date and a type. See [`readiness`] for the rule.
    Balanced,
    /// Strict, except that a reading whose only doubt is which side its
    /// parties are on ([`ReviewReason::PartyRoleUnconfirmed`]) is ready:
    /// every party in the name is in the document, the date, the type and
    /// the token probabilities passed, and only the joining word rests on
    /// the reply's roles. Off unless a settings file names it; Settings does
    /// not offer it. See [`readiness`] for the measurement behind it.
    #[serde(rename = "strict_sides")]
    StrictSides,
}

impl ReviewLevel {
    pub const ALL: [Self; 3] = [Self::Strict, Self::Balanced, Self::StrictSides];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Balanced => "balanced",
            Self::StrictSides => "strict_sides",
        }
    }

    /// The level `word` names, exactly as [`Self::as_str`] writes it.
    pub fn parse(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.as_str() == word)
    }

    /// Whether this is the default level. A record or a report made at it
    /// leaves the level out, so it is written as it was before the levels.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn is_strict(&self) -> bool {
        *self == Self::Strict
    }
}

/// A stored level this build does not know reads as strict, whatever it is:
/// a word a later build wrote, a typing mistake, a value that is not a word.
/// Strict files the least on its own, so it is the safe reading of a setting
/// that cannot be understood, and one unknown word must not make a whole
/// settings file unreadable.
impl<'de> Deserialize<'de> for ReviewLevel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Word(String),
            Other(serde::de::IgnoredAny),
        }
        Ok(match Stored::deserialize(deserializer)? {
            Stored::Word(word) => Self::parse(&word).unwrap_or_default(),
            Stored::Other(_) => Self::Strict,
        })
    }
}

/// What a review level makes of one reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Readiness {
    /// Whether the reading is ready at the level.
    pub ready: bool,
    /// The reading's own reasons that hold it at the level, in the
    /// reading's order. Empty for a ready reading. A waiting reading can
    /// have none too: one with no date or no type that no reason speaks of,
    /// or one the engine did not mark ready and gave no reason for.
    pub held_by: Vec<ReviewReason>,
}

impl Readiness {
    /// The answer as a status: what a record made of the reading starts
    /// from, before the queue's own holds.
    pub const fn status(&self) -> ProposalStatus {
        if self.ready {
            ProposalStatus::Ready
        } else {
            ProposalStatus::NeedsReview
        }
    }

    const fn ready() -> Self {
        Self {
            ready: true,
            held_by: Vec::new(),
        }
    }
}

/// The reasons that are minor at every level, strict included: each leaves
/// the facts the name is made of standing, or names one the queue has
/// settled, under the conditions [`readiness`] gives. Everything that
/// touches the date's correctness, a party, the type the reply gave, the
/// model's figures, the quick model, the parser and the queue's own holds
/// is not here, and holds.
///
/// * [`ReviewReason::DateMissing`], once the reading has a date its name
///   starts with. The engine raises it only on a reading with no date, so
///   a date beside it is one the queue gave it from the file itself - the
///   original filename's, or the file's last-modified date - and the record
///   says which ([`supplied_date`]). With no such date it holds.
/// * [`ReviewReason::TypeInferred`]: the type is the document's own title,
///   or the kind its layout shows ("Letter", "Email"). It holds where the
///   name would say "Document", and where a title stood in for a type the
///   document did not support (clause 5 of [`readiness`]).
/// * [`ReviewReason::DescriptionInvalid`] and
///   [`ReviewReason::DescriptionUnsupported`]: the description is not in
///   the name. The evidence pipeline composes it from validated facts only,
///   and leaves out any it could not find; a sentence the model wrote
///   itself that says more than the document is not kept by the queue.
pub const MINOR: [ReviewReason; 4] = [
    ReviewReason::DateMissing,
    ReviewReason::TypeInferred,
    ReviewReason::DescriptionInvalid,
    ReviewReason::DescriptionUnsupported,
];

/// Whether `analysis` is ready at `level`, and which of its reasons hold it.
///
/// A reading the engine marked ready is ready at every level. Any other
/// reading is ready when all of these hold, and waits otherwise.
///
/// 1. It has a date: the proposal's date is there and the name starts with
///    it.
/// 2. Every reason on it is minor ([`MINOR`]) or, at balanced, a note:
///    [`ReviewReason::TypeMissing`], [`ReviewReason::TypeUnsupported`] or
///    [`ReviewReason::LowConfidence`]. [`ReviewReason::PartyRoleUnconfirmed`]
///    holds at both levels: as a note at balanced it filed 89 more names on
///    real sets 1 to 8, 19 of them really wrong (November 2026). At
///    strict_sides it does not hold: with every other doubt held as at
///    strict, it was the only hold on 19 names on the same sets, 1 of them
///    really wrong (a date), 5 minor (the joining word). Any other reason holds it: every date
///    reason but a missing date the queue has filled in, every party
///    reason, a parser warning, the model's own request for review, the
///    quick model's readings ([`ReviewReason::FastModelReading`]), at strict
///    a low confidence and a type the reply gave that the document does not
///    confirm, and any reason added after this was written. The lists name
///    what does not hold, so a new reason holds until someone decides
///    otherwise.
/// 3. A [`ReviewReason::LowConfidence`] at balanced is there for the token
///    probabilities only. When the model's own reported confidence is below
///    [`READY_CONFIDENCE`], or the reading stores no reply to read it from,
///    it waits.
/// 4. The proposal has a document type. A name that would say "Document"
///    waits, whatever the proposal's type is: none, the word itself, or the
///    word with the file's extension after it, which a name leaves out. So
///    does a name that would say "Document" and an ending that looks like
///    another extension, "Document.docx" on a PDF.
/// 5. A title did not stand in for a type the document did not support:
///    [`ReviewReason::TypeUnsupported`] and [`ReviewReason::TypeInferred`]
///    together, without [`ReviewReason::TypeMissing`], wait.
///
/// A reading the engine did not mark ready and gave no reason for waits at
/// both levels: nothing says what is in doubt, so nothing says it is safe.
///
/// The queue's own holds are not decided here: a near duplicate, a learned
/// spelling nobody has checked, a changed file and the rest are applied
/// after this, at every level.
pub fn readiness(analysis: &DocumentAnalysis, level: ReviewLevel) -> Readiness {
    if analysis.status == ProposalStatus::Ready {
        return Readiness::ready();
    }
    let reasons = &analysis.review_reasons;
    let balanced = level == ReviewLevel::Balanced;
    let sides = level == ReviewLevel::StrictSides;
    let has = |reason: ReviewReason| reasons.contains(&reason);
    let typed = names_a_type(analysis);
    let dated = dated(analysis);
    let title_stood_in = has(ReviewReason::TypeUnsupported)
        && has(ReviewReason::TypeInferred)
        && !has(ReviewReason::TypeMissing);
    let unsure_of_itself = analysis
        .model_proposal
        .as_ref()
        .is_none_or(|reply| !reply.confidence.is_finite() || reply.confidence < READY_CONFIDENCE);
    let held_by = reasons
        .iter()
        .copied()
        .filter(|reason| match reason {
            // Minor, at every level.
            ReviewReason::DateMissing => !dated,
            ReviewReason::TypeInferred => !typed || title_stood_in,
            ReviewReason::DescriptionInvalid | ReviewReason::DescriptionUnsupported => false,
            // Notes, at balanced only.
            ReviewReason::TypeMissing if balanced => !typed,
            ReviewReason::TypeUnsupported if balanced => !typed || title_stood_in,
            ReviewReason::LowConfidence if balanced => unsure_of_itself,
            // The side alone, at strict_sides only.
            ReviewReason::PartyRoleUnconfirmed if sides => false,
            _ => true,
        })
        .collect::<Vec<_>>();
    Readiness {
        ready: held_by.is_empty() && !reasons.is_empty() && typed && dated,
        held_by,
    }
}

/// The date the queue gave a reading the document gave none, when it has
/// one: a date beside [`ReviewReason::DateMissing`], which the engine
/// raises only on a reading with no date. The queue takes it from the file
/// itself, and the record says from where; nothing in the document backs
/// it, so whatever compares a reading's date with another document's -
/// whether two scans are one document - reads the reading as undated.
pub fn supplied_date(analysis: &DocumentAnalysis) -> Option<&str> {
    analysis
        .proposal
        .document_date
        .as_deref()
        .filter(|date| !date.trim().is_empty())
        .filter(|_| analysis.review_reasons.contains(&ReviewReason::DateMissing))
}

/// Whether the reading has a date its name starts with.
fn dated(analysis: &DocumentAnalysis) -> bool {
    analysis
        .proposal
        .document_date
        .as_deref()
        .map(str::trim)
        .is_some_and(|date| !date.is_empty() && analysis.filename.starts_with(date))
}

/// What each doubt on a record leaves of how sure Intern is of its name:
/// nine tenths ([`name_confidence`]).
pub const DOUBT_FACTOR: f32 = 0.9;

/// The most a reading that waits for a person is shown as: one in two
/// ([`name_confidence`]). Below any ready reading's figure unless the model
/// itself put that one lower.
pub const WAITING_CEILING: f32 = 0.5;

/// How sure Intern is of a reading's name, from 0 to 1: the figure a person
/// is shown beside it, and the one place it is worked out.
///
/// 1. The lower of the model's two figures: its own rating of the reading
///    (the proposal's confidence; the evidence pipeline's reply carries no
///    rating and reads as sure), and the least probable token of its date
///    and its parties' names ([`crate::TokenConfidence::min`]). A reading
///    with no token probabilities - a hosted model's, or one stored before
///    they were kept - has its rating alone.
/// 2. A reading that `waits` for a person is shown no surer than
///    [`WAITING_CEILING`], whatever the model's figures say: the model was
///    sure of something a check then doubted, and the window says the
///    check's word.
/// 3. Each of the record's `doubts` - its review reasons, those that hold
///    it and those that are only notes - leaves [`DOUBT_FACTOR`] of what
///    is left.
///
/// So a ready reading with nothing on it shows the model's own figure, a
/// ready one with a note a little less, and one in review at most 45%
/// with a reason on it, less with each further reason. A figure that is
/// not a number reads as 0.
pub fn name_confidence(analysis: &DocumentAnalysis, doubts: usize, waits: bool) -> f32 {
    let rating = analysis.proposal.confidence;
    let tokens = analysis.token_confidence.map_or(1.0, |tokens| tokens.min);
    let mut sure = if rating.is_finite() && tokens.is_finite() {
        rating.min(tokens).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if waits {
        sure = sure.min(WAITING_CEILING);
    }
    let doubts = i32::try_from(doubts).unwrap_or(i32::MAX);
    sure * DOUBT_FACTOR.powi(doubts)
}

/// Whether the reading has a document type a name can say: one that is
/// there, and that says more than the word a name says when there is none.
///
/// The type is read the way the reading's name was composed
/// ([`type_segment`]), with the file's extension. A reply sometimes ends
/// the type with it, and the name leaves it out: "Document.pdf" on a PDF is
/// named "Document". Read with no extension, that type counted as one.
///
/// A name keeps any other ending that looks like an extension
/// ([`without_extension_like_endings`]): "Document.docx" on a PDF is named
/// "Document.docx.pdf", and so is "Document.docx." with a period after it.
/// Such a name says no more about the document than "Document" does, so
/// that type is no type either.
///
/// An ending cannot be told from a short word. A period and one to five
/// letters or digits is read as an ending whatever it spells, so
/// "Document.Memo", "Document.Draft" and "Document.2024" are no type and
/// wait. A type keeps the words it has otherwise: "Document Retention
/// Policy"; "Document.Review Memo", where what follows the period has a
/// space in it; and "Document.Policy" on a PDF, six letters. On a file
/// with no extension that last one waits too: the name's own ending is
/// read as the file's extension there, at any length ([`extension_of`]).
fn names_a_type(analysis: &DocumentAnalysis) -> bool {
    let extension = extension_of(&analysis.filename);
    analysis
        .proposal
        .document_type
        .as_deref()
        .is_some_and(|stated| {
            let named = type_segment(Some(stated), extension, Spelling::Document);
            let words = without_extension_like_endings(&named);
            !words.is_empty() && !words.eq_ignore_ascii_case(DEFAULT_TYPE)
        })
}

/// The most letters or digits an ending can have and still look like an
/// extension: "docx", "xlsx", "jpeg" and "pages" do, "Review" does not.
const EXTENSION_LIKE_MAX: usize = 5;

/// `segment` without the periods and spaces it ends with, and without the
/// endings that look like an extension: a period and one to
/// [`EXTENSION_LIKE_MAX`] letters or digits. Every one of them, as a name drops
/// the file's own extension from a type however often the type repeats it.
/// Nothing is left of a segment that is only such endings.
fn without_extension_like_endings(segment: &str) -> &str {
    let mut words = segment;
    loop {
        words = words.trim_end_matches(['.', ' ']);
        match words.rsplit_once('.') {
            Some((before, ending))
                if (1..=EXTENSION_LIKE_MAX).contains(&ending.len())
                    && ending.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                words = before;
            }
            _ => return words,
        }
    }
}

/// What follows the last period of `filename`, when that is letters and
/// digits only, and nothing otherwise. A composed name ends with the file's
/// extension when the file has one ([`crate::naming::compose_filename`]),
/// and then this is that extension. A file with none is named with none,
/// and then this is whatever the name happens to end with after a period:
/// "2" for a name that ends "v1.2". [`names_a_type`] only takes it off the
/// end of the type, as the name did, so a wrong one can only shorten a type
/// that ends the same way. That can make it no type, and never makes a type
/// of what is none.
fn extension_of(filename: &str) -> &str {
    filename
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| extension.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AnalysisTelemetry, ModelProposal, ValidatedProposal};
    use crate::naming::compose_filename;

    const DATE: &str = "2026-01-16";

    /// A finished reading with a date, a type and a reply that reported
    /// itself sure, carrying `reasons`: ready by the engine when there are
    /// none.
    fn reading(reasons: &[ReviewReason]) -> DocumentAnalysis {
        DocumentAnalysis {
            filename: format!("{DATE} Dispatch Note.pdf"),
            description: "A dispatch note.".into(),
            status: if reasons.is_empty() {
                ProposalStatus::Ready
            } else {
                ProposalStatus::NeedsReview
            },
            review_reasons: reasons.to_vec(),
            proposal: ValidatedProposal {
                document_type: Some("Dispatch Note".into()),
                document_date: Some(DATE.into()),
                confidence: 0.9,
                ..ValidatedProposal::default()
            },
            telemetry: AnalysisTelemetry::default(),
            model_proposal: Some(ModelProposal {
                confidence: 0.9,
                ..ModelProposal::default()
            }),
            stated_dates: Vec::new(),
            text_fingerprint: None,
            token_confidence: None,
            facts: None,
        }
    }

    fn strict(analysis: &DocumentAnalysis) -> Readiness {
        readiness(analysis, ReviewLevel::Strict)
    }

    fn balanced(analysis: &DocumentAnalysis) -> Readiness {
        readiness(analysis, ReviewLevel::Balanced)
    }

    fn waits(held_by: &[ReviewReason]) -> Readiness {
        Readiness {
            ready: false,
            held_by: held_by.to_vec(),
        }
    }

    /// The reasons that are notes at balanced and hold at strict.
    const NOTES: [ReviewReason; 3] = [
        ReviewReason::TypeMissing,
        ReviewReason::TypeUnsupported,
        ReviewReason::LowConfidence,
    ];

    /// Every reason there is. [`listed`] does not compile until a new one
    /// is added here, so a new reason is tested as holding until someone
    /// decides otherwise.
    const EVERY_REASON: [ReviewReason; 19] = [
        ReviewReason::DateMissing,
        ReviewReason::DateUnsupported,
        ReviewReason::TypeMissing,
        ReviewReason::TypeUnsupported,
        ReviewReason::TypeInferred,
        ReviewReason::PartyUnsupported,
        ReviewReason::PartyMissing,
        ReviewReason::DescriptionUnsupported,
        ReviewReason::DescriptionInvalid,
        ReviewReason::LowConfidence,
        ReviewReason::ModelRequestedReview,
        ReviewReason::ParserWarning,
        ReviewReason::DateImplausible,
        ReviewReason::DateIsDeadline,
        ReviewReason::DateAmbiguous,
        ReviewReason::DateIsPeriod,
        ReviewReason::DateIsFormula,
        ReviewReason::FastModelReading,
        ReviewReason::PartyRoleUnconfirmed,
    ];

    fn listed(reason: ReviewReason) -> bool {
        match reason {
            ReviewReason::DateMissing
            | ReviewReason::DateUnsupported
            | ReviewReason::TypeMissing
            | ReviewReason::TypeUnsupported
            | ReviewReason::TypeInferred
            | ReviewReason::PartyUnsupported
            | ReviewReason::PartyMissing
            | ReviewReason::DescriptionUnsupported
            | ReviewReason::DescriptionInvalid
            | ReviewReason::LowConfidence
            | ReviewReason::ModelRequestedReview
            | ReviewReason::ParserWarning
            | ReviewReason::DateImplausible
            | ReviewReason::DateIsDeadline
            | ReviewReason::DateAmbiguous
            | ReviewReason::DateIsPeriod
            | ReviewReason::DateIsFormula
            | ReviewReason::FastModelReading
            | ReviewReason::PartyRoleUnconfirmed => EVERY_REASON.contains(&reason),
        }
    }

    /// Strict is the engine's own status but for the minor reasons: a
    /// reading the engine marked ready is ready, and one it held waits for
    /// every reason that is not minor, held by exactly those.
    #[test]
    fn strict_is_the_engines_own_status_but_for_the_minor_reasons() {
        let mut readings = vec![reading(&[])];
        readings.extend(EVERY_REASON.iter().map(|reason| reading(&[*reason])));
        // A stored reading need not be one the engine would make now.
        let mut ready_with_a_reason = reading(&[ReviewReason::PartyUnsupported]);
        ready_with_a_reason.status = ProposalStatus::Ready;
        let mut waiting_with_none = reading(&[]);
        waiting_with_none.status = ProposalStatus::NeedsReview;
        readings.extend([ready_with_a_reason, waiting_with_none]);
        for analysis in &readings {
            let answer = strict(analysis);
            let minor_only = !analysis.review_reasons.is_empty()
                && analysis
                    .review_reasons
                    .iter()
                    .all(|reason| MINOR.contains(reason));
            assert_eq!(
                answer.ready,
                analysis.status == ProposalStatus::Ready || minor_only,
                "{analysis:?}"
            );
            if answer.ready {
                assert!(answer.held_by.is_empty());
            } else {
                let held = analysis
                    .review_reasons
                    .iter()
                    .copied()
                    .filter(|reason| !MINOR.contains(reason))
                    .collect::<Vec<_>>();
                assert_eq!(answer.held_by, held, "{analysis:?}");
            }
        }
    }

    /// The minor reasons are ready at every level, alone and together, and
    /// beside a reason that holds they hold nothing themselves.
    #[test]
    fn the_minor_reasons_hold_at_no_level() {
        for level in ReviewLevel::ALL {
            for minor in MINOR {
                assert_eq!(
                    readiness(&reading(&[minor]), level),
                    Readiness::ready(),
                    "{minor:?} at {level:?}"
                );
            }
            assert_eq!(
                readiness(&reading(&MINOR), level),
                Readiness::ready(),
                "{level:?}"
            );
            assert_eq!(
                readiness(
                    &reading(&[
                        ReviewReason::DescriptionInvalid,
                        ReviewReason::PartyMissing,
                        ReviewReason::TypeInferred,
                    ]),
                    level
                ),
                waits(&[ReviewReason::PartyMissing]),
                "{level:?}"
            );
        }
        // What does not touch the name's facts, and nothing more.
        for reason in [
            ReviewReason::DateUnsupported,
            ReviewReason::DateAmbiguous,
            ReviewReason::DateIsDeadline,
            ReviewReason::DateImplausible,
            ReviewReason::DateIsPeriod,
            ReviewReason::DateIsFormula,
            ReviewReason::PartyUnsupported,
            ReviewReason::PartyMissing,
            ReviewReason::PartyRoleUnconfirmed,
            ReviewReason::ParserWarning,
            ReviewReason::ModelRequestedReview,
            ReviewReason::FastModelReading,
        ] {
            assert!(!MINOR.contains(&reason), "{reason:?}");
        }
    }

    /// At strict_sides the side alone does not hold, and nothing else
    /// changes: beside any other doubt it still waits for that doubt, and
    /// the other levels still hold it.
    #[test]
    fn strict_sides_files_a_name_whose_only_doubt_is_the_side() {
        let side = reading(&[ReviewReason::PartyRoleUnconfirmed]);
        assert_eq!(
            readiness(&side, ReviewLevel::StrictSides),
            Readiness::ready()
        );
        for level in [ReviewLevel::Strict, ReviewLevel::Balanced] {
            assert_eq!(
                readiness(&side, level),
                waits(&[ReviewReason::PartyRoleUnconfirmed]),
                "{level:?}"
            );
        }
        for other in [
            ReviewReason::LowConfidence,
            ReviewReason::TypeMissing,
            ReviewReason::TypeUnsupported,
            ReviewReason::PartyUnsupported,
            ReviewReason::DateAmbiguous,
        ] {
            let both = reading(&[ReviewReason::PartyRoleUnconfirmed, other]);
            assert_eq!(
                readiness(&both, ReviewLevel::StrictSides),
                waits(&[other]),
                "{other:?}"
            );
        }
        assert_eq!(
            ReviewLevel::parse("strict_sides"),
            Some(ReviewLevel::StrictSides)
        );
        assert_eq!(
            serde_json::to_value(ReviewLevel::StrictSides).unwrap(),
            serde_json::json!("strict_sides")
        );
    }

    /// A missing date is minor only once the queue has given the reading a
    /// date its name starts with ([`supplied_date`]). With none it holds,
    /// as before, and so does any other doubt about the date beside it.
    #[test]
    fn a_missing_date_is_minor_only_with_a_date_from_the_file() {
        let supplied = reading(&[ReviewReason::DateMissing]);
        assert_eq!(supplied_date(&supplied), Some(DATE));
        for level in ReviewLevel::ALL {
            assert_eq!(readiness(&supplied, level), Readiness::ready(), "{level:?}");

            let mut none = supplied.clone();
            none.proposal.document_date = None;
            none.filename = "Dispatch Note.pdf".into();
            assert_eq!(supplied_date(&none), None);
            assert_eq!(
                readiness(&none, level),
                waits(&[ReviewReason::DateMissing]),
                "{level:?}"
            );

            let mut named_without_it = supplied.clone();
            named_without_it.filename = "Dispatch Note.pdf".into();
            assert_eq!(
                readiness(&named_without_it, level),
                waits(&[ReviewReason::DateMissing]),
                "{level:?}"
            );

            let doubted = reading(&[ReviewReason::DateMissing, ReviewReason::DateAmbiguous]);
            assert_eq!(
                readiness(&doubted, level),
                waits(&[ReviewReason::DateAmbiguous]),
                "{level:?}"
            );
        }
        // A date the document gave is no date the queue supplied.
        assert_eq!(supplied_date(&reading(&[])), None);
        assert_eq!(
            supplied_date(&reading(&[ReviewReason::DateAmbiguous])),
            None
        );
    }

    /// Balanced only ever adds to what strict files: never the reverse,
    /// even for a stored reading the rule would otherwise hold.
    #[test]
    fn ready_at_strict_is_ready_at_balanced() {
        let plain = reading(&[]);
        assert_eq!(strict(&plain), Readiness::ready());
        assert_eq!(balanced(&plain), Readiness::ready());

        let mut bare = reading(&[]);
        bare.proposal.document_date = None;
        bare.proposal.document_type = None;
        bare.model_proposal = None;
        bare.filename = "Document.pdf".into();
        let mut noted = reading(&[ReviewReason::PartyUnsupported]);
        noted.status = ProposalStatus::Ready;
        for analysis in [bare, noted] {
            assert!(strict(&analysis).ready);
            assert_eq!(balanced(&analysis), Readiness::ready(), "{analysis:?}");
        }
    }

    /// Clause 2: the three reasons about the type the reply gave and the
    /// token probabilities are notes at balanced, alone, together and with
    /// the minor ones, and every other reason holds.
    #[test]
    fn only_the_named_reasons_are_notes_at_balanced() {
        for note in NOTES {
            let analysis = reading(&[note]);
            assert_eq!(strict(&analysis), waits(&[note]));
            assert_eq!(balanced(&analysis), Readiness::ready(), "{note:?}");
        }
        let together = reading(&[
            ReviewReason::TypeMissing,
            ReviewReason::TypeInferred,
            ReviewReason::DescriptionUnsupported,
            ReviewReason::LowConfidence,
        ]);
        assert_eq!(balanced(&together), Readiness::ready());
        assert_eq!(
            strict(&together),
            waits(&[ReviewReason::TypeMissing, ReviewReason::LowConfidence])
        );

        for reason in EVERY_REASON {
            if NOTES.contains(&reason) || MINOR.contains(&reason) {
                continue;
            }
            assert_eq!(
                balanced(&reading(&[reason])),
                waits(&[reason]),
                "{reason:?}"
            );
            // Beside a note it still holds, and the note does not.
            assert_eq!(
                balanced(&reading(&[ReviewReason::TypeUnsupported, reason])),
                waits(&[reason]),
                "{reason:?}"
            );
        }
    }

    /// A reason this rule has not heard of holds. Every reason that is not
    /// minor or a note is tried, and the list of every reason cannot fall
    /// behind the enum.
    #[test]
    fn an_unknown_reason_holds() {
        assert!(EVERY_REASON.into_iter().all(listed));
        let holding = |level| {
            EVERY_REASON
                .into_iter()
                .filter(|reason| !readiness(&reading(&[*reason]), level).ready)
                .count()
        };
        assert_eq!(
            holding(ReviewLevel::Balanced),
            EVERY_REASON.len() - NOTES.len() - MINOR.len()
        );
        assert_eq!(
            holding(ReviewLevel::Strict),
            EVERY_REASON.len() - MINOR.len()
        );
        for reason in NOTES.into_iter().chain(MINOR) {
            assert!(EVERY_REASON.contains(&reason));
        }
    }

    /// The quick model's readings wait at every level, whatever else they
    /// say.
    #[test]
    fn the_quick_models_readings_wait_at_every_level() {
        let mut quick = reading(&[]);
        crate::tiers::hold_fast_reading(&mut quick);
        let mut noted = reading(&[ReviewReason::TypeInferred]);
        crate::tiers::hold_fast_reading(&mut noted);
        for level in ReviewLevel::ALL {
            assert_eq!(
                readiness(&quick, level),
                waits(&[ReviewReason::FastModelReading]),
                "{level:?}"
            );
            assert!(!readiness(&noted, level).ready, "{level:?}");
        }
        assert_eq!(balanced(&noted), waits(&[ReviewReason::FastModelReading]));
    }

    /// Clause 1: no date, or a name that does not start with it, waits.
    #[test]
    fn a_reading_with_no_date_waits_at_balanced() {
        let dated = reading(&[ReviewReason::TypeInferred]);
        assert!(balanced(&dated).ready);

        let mut undated = dated.clone();
        undated.proposal.document_date = None;
        undated.filename = "Dispatch Note.pdf".into();
        assert_eq!(balanced(&undated), waits(&[]));

        let mut blank = dated.clone();
        blank.proposal.document_date = Some("  ".into());
        blank.filename = "Dispatch Note.pdf".into();
        assert!(!balanced(&blank).ready);

        let mut named_without_it = dated;
        named_without_it.filename = "Dispatch Note.pdf".into();
        assert_eq!(balanced(&named_without_it), waits(&[]));
    }

    /// Clause 3: a low confidence is a note only when it is the token
    /// probabilities'. The model's own figure below the engine's line
    /// holds, and so does a reading with no reply to read the figure from.
    #[test]
    fn low_confidence_is_a_note_only_when_it_is_the_tokens() {
        let with = |confidence: Option<f32>| {
            let mut analysis = reading(&[ReviewReason::LowConfidence]);
            analysis.model_proposal = confidence.map(|confidence| ModelProposal {
                confidence,
                ..ModelProposal::default()
            });
            balanced(&analysis)
        };
        let held = waits(&[ReviewReason::LowConfidence]);
        assert_eq!(with(Some(0.9)), Readiness::ready());
        assert_eq!(with(Some(READY_CONFIDENCE)), Readiness::ready());
        assert_eq!(with(Some(READY_CONFIDENCE - 0.01)), held);
        assert_eq!(with(Some(0.0)), held);
        assert_eq!(with(Some(f32::NAN)), held);
        assert_eq!(with(None), held);

        // The figure only matters to a reading that carries the reason.
        let mut other = reading(&[ReviewReason::TypeInferred]);
        other.model_proposal = None;
        assert!(balanced(&other).ready);
    }

    /// Clause 4: a reading with no type, whose name would say "Document",
    /// waits. A title standing in for a type the reply did not give is a
    /// type.
    #[test]
    fn a_reading_with_no_type_waits_at_balanced() {
        let mut untyped = reading(&[ReviewReason::TypeMissing]);
        untyped.proposal.document_type = None;
        untyped.filename = format!("{DATE} Document.pdf");
        assert_eq!(balanced(&untyped), waits(&[ReviewReason::TypeMissing]));

        // No type, and no reason that says so: it waits all the same.
        let mut unexplained = reading(&[ReviewReason::DescriptionUnsupported]);
        unexplained.proposal.document_type = None;
        unexplained.filename = format!("{DATE} Document.pdf");
        assert_eq!(balanced(&unexplained), waits(&[]));

        // A type that is no type: the name says "Document" either way.
        for word in ["", "  ", "Document", "DOCUMENT", "???"] {
            let mut analysis = reading(&[ReviewReason::TypeInferred]);
            analysis.proposal.document_type = Some(word.into());
            analysis.filename = format!("{DATE} Document.pdf");
            assert_eq!(
                balanced(&analysis),
                waits(&[ReviewReason::TypeInferred]),
                "{word:?}"
            );
        }
        let mut kept = reading(&[ReviewReason::TypeInferred]);
        kept.proposal.document_type = Some("Document Retention Policy".into());
        kept.filename = format!("{DATE} Document Retention Policy.pdf");
        assert!(balanced(&kept).ready);

        // A reply sometimes ends the type with the file's extension, and
        // the name drops it: "Document.pdf" on a PDF is named "Document".
        for word in ["Document.pdf", "document.PDF", "Document.pdf.pdf", ".pdf"] {
            let mut analysis = reading(&[ReviewReason::TypeUnsupported]);
            analysis.proposal.document_type = Some(word.into());
            analysis.filename = compose_filename(&analysis.proposal, "pdf", &[]).value;
            assert!(
                analysis
                    .filename
                    .eq_ignore_ascii_case(&format!("{DATE} Document.pdf")),
                "{word:?} is named {}",
                analysis.filename
            );
            assert_eq!(
                balanced(&analysis),
                waits(&[ReviewReason::TypeUnsupported]),
                "{word:?}"
            );
        }
        // The file's own extension is dropped whatever its length, also
        // where it is too long to be taken for an ending that looks like
        // one.
        let mut long = reading(&[ReviewReason::TypeUnsupported]);
        long.proposal.document_type = Some("Document.markdown".into());
        long.filename = compose_filename(&long.proposal, "markdown", &[]).value;
        assert_eq!(long.filename, format!("{DATE} Document.markdown"));
        assert_eq!(balanced(&long), waits(&[ReviewReason::TypeUnsupported]));

        // A type that says more than "Document" is a type with the
        // extension dropped too.
        let mut analysis = reading(&[ReviewReason::TypeUnsupported]);
        analysis.proposal.document_type = Some("Dispatch Note.pdf".into());
        analysis.filename = compose_filename(&analysis.proposal, "pdf", &[]).value;
        assert_eq!(analysis.filename, format!("{DATE} Dispatch Note.pdf"));
        assert_eq!(balanced(&analysis), Readiness::ready());

        let stood_in = reading(&[ReviewReason::TypeMissing, ReviewReason::TypeInferred]);
        assert_eq!(balanced(&stood_in), Readiness::ready());
    }

    /// A reading of a PDF whose reply gave the type `word`, with a note
    /// about the type, named as the engine names it.
    fn typed(word: &str) -> DocumentAnalysis {
        let mut analysis = reading(&[ReviewReason::TypeUnsupported]);
        analysis.proposal.document_type = Some(word.into());
        analysis.filename = compose_filename(&analysis.proposal, "pdf", &[]).value;
        analysis
    }

    /// Clause 4, where the type is "Document" with an ending that looks
    /// like an extension and is not the file's: another kind of file's, or
    /// the file's own with a period after it. The name keeps such an
    /// ending, "Document.docx.pdf", and says no more about the document
    /// than "Document" does: it waits. So does "Document" with a short
    /// word after a period, "Document.Memo": one to five letters or digits
    /// there are read as an ending whatever they spell. A type with other
    /// words of its own is a type.
    #[test]
    fn a_type_that_is_document_with_an_ending_like_an_extension_waits_at_balanced() {
        for (word, named) in [
            ("Document.docx", "Document.docx"),
            ("Document.pdf.", "Document.pdf"),
            ("Document.docx.", "Document.docx"),
            ("Document.docx.pdf", "Document.docx"),
            ("Document .docx", "Document .docx"),
            ("Document..docx", "Document..docx"),
            ("Document.docx.xlsx", "Document.docx.xlsx"),
            ("Document.c", "Document.c"),
            ("Document.xhtml", "Document.xhtml"),
            ("Document.12345", "Document.12345"),
            (".docx", ".docx"),
            // A short word after a period is read as an ending too.
            ("Document.Memo", "Document.Memo"),
            ("Document.Draft", "Document.Draft"),
            ("Document.2024", "Document.2024"),
        ] {
            let analysis = typed(word);
            assert_eq!(analysis.filename, format!("{DATE} {named}.pdf"), "{word:?}");
            assert_eq!(
                balanced(&analysis),
                waits(&[ReviewReason::TypeUnsupported]),
                "{word:?}"
            );
        }
        // Whatever its capitals.
        for word in ["DOCUMENT.DOCX", "document.Docx"] {
            assert_eq!(
                balanced(&typed(word)),
                waits(&[ReviewReason::TypeUnsupported]),
                "{word:?}"
            );
        }
        // Words of its own before or after, and an ending too long or too
        // odd to be an extension.
        for word in [
            "Document Retention Policy",
            "Document.Review Memo",
            "Document v1.2",
            "Document No. 5",
            "Document.Policy",
            "Document.a-b",
            "Documents.docx",
            "Board Document.docx",
        ] {
            assert_eq!(balanced(&typed(word)), Readiness::ready(), "{word:?}");
        }
        // A file with no extension is named with none. Words after a
        // period and a space in its name are words of the type.
        let bare = |word: &str| {
            let mut analysis = reading(&[ReviewReason::TypeUnsupported]);
            analysis.proposal.document_type = Some(word.into());
            analysis.filename = compose_filename(&analysis.proposal, "", &[]).value;
            assert_eq!(analysis.filename, format!("{DATE} {word}"));
            analysis
        };
        assert_eq!(balanced(&bare("Document. Annex")), Readiness::ready());
        // Letters and digits right after the last period of such a name
        // are read as the file's extension, at any length, and taken off
        // the type: "Document.Policy", a type on a PDF, waits here.
        assert_eq!(
            balanced(&bare("Document.Policy")),
            waits(&[ReviewReason::TypeUnsupported])
        );
    }

    /// Clause 5: a title that stood in for a type the document did not
    /// support waits. Either reason alone does not, nor both where the
    /// reply gave no type at all.
    #[test]
    fn a_title_that_stood_in_for_an_unsupported_type_waits_at_balanced() {
        let stood_in = reading(&[ReviewReason::TypeUnsupported, ReviewReason::TypeInferred]);
        assert_eq!(
            balanced(&stood_in),
            waits(&[ReviewReason::TypeUnsupported, ReviewReason::TypeInferred])
        );
        let beside_a_note = reading(&[
            ReviewReason::TypeUnsupported,
            ReviewReason::DescriptionUnsupported,
            ReviewReason::TypeInferred,
        ]);
        assert_eq!(
            balanced(&beside_a_note),
            waits(&[ReviewReason::TypeUnsupported, ReviewReason::TypeInferred])
        );
        for reasons in [
            &[ReviewReason::TypeUnsupported][..],
            &[ReviewReason::TypeInferred][..],
            &[
                ReviewReason::TypeMissing,
                ReviewReason::TypeUnsupported,
                ReviewReason::TypeInferred,
            ][..],
        ] {
            assert_eq!(
                balanced(&reading(reasons)),
                Readiness::ready(),
                "{reasons:?}"
            );
        }
    }

    /// A reading the engine did not mark ready and gave no reason for waits
    /// at both levels.
    #[test]
    fn a_reading_held_for_no_stated_reason_waits_at_both_levels() {
        let mut analysis = reading(&[]);
        analysis.status = ProposalStatus::NeedsReview;
        for level in ReviewLevel::ALL {
            assert_eq!(readiness(&analysis, level), waits(&[]), "{level:?}");
        }
    }

    fn close(left: f32, right: f32) -> bool {
        (left - right).abs() < 1e-6
    }

    /// A reading `rated` by the model, with `tokens` as its least probable
    /// date or party token when the model reported any.
    fn rated(rating: f32, tokens: Option<f32>) -> DocumentAnalysis {
        let mut analysis = reading(&[]);
        analysis.proposal.confidence = rating;
        analysis.token_confidence = tokens.map(|min| crate::domain::TokenConfidence {
            min,
            mean: min,
            tokens: 12,
        });
        analysis
    }

    /// The figure shown is the lower of the model's rating and its least
    /// token probability. The evidence pipeline's reply has no rating and
    /// reads as sure, so its token figure is what shows; a reading with no
    /// token figures, a hosted model's or one stored before they were
    /// kept, shows its rating.
    #[test]
    fn the_figure_shown_is_the_lower_of_the_models_two() {
        assert!(close(
            name_confidence(&rated(1.0, Some(0.83)), 0, false),
            0.83
        ));
        assert!(close(
            name_confidence(&rated(0.7, Some(0.95)), 0, false),
            0.7
        ));
        assert!(close(name_confidence(&rated(0.9, None), 0, false), 0.9));
        // What is not a figure counts for nothing, and what is out of
        // range is brought into it.
        assert!(close(
            name_confidence(&rated(f32::NAN, Some(0.9)), 0, false),
            0.0
        ));
        assert!(close(
            name_confidence(&rated(1.0, Some(f32::NAN)), 0, false),
            0.0
        ));
        assert!(close(name_confidence(&rated(1.7, None), 0, false), 1.0));
        assert!(close(name_confidence(&rated(-0.2, None), 0, false), 0.0));
    }

    /// A reading in review never shows above one in two, and each reason
    /// on the record takes a tenth of what is left: so a reading the model
    /// was sure of, held for a party, reads 45% where it used to read
    /// 100%. A ready reading with nothing on it shows its figure exactly,
    /// and one with a note a little less.
    #[test]
    fn every_doubt_pulls_the_figure_down_and_review_never_reads_as_sure() {
        let sure = rated(1.0, Some(1.0));
        assert!(close(name_confidence(&sure, 0, false), 1.0));
        assert!(close(name_confidence(&sure, 1, false), 0.9));
        assert!(close(name_confidence(&sure, 0, true), WAITING_CEILING));
        assert!(close(name_confidence(&sure, 1, true), 0.45));
        assert!(close(name_confidence(&sure, 2, true), 0.405));
        // Below the ceiling already, review takes off only the reasons.
        let unsure = rated(1.0, Some(0.3));
        assert!(close(name_confidence(&unsure, 1, true), 0.27));
        // However sure the model and however few the reasons, a reading in
        // review shows below every ready reading the model was as sure of.
        for doubts in 0..6 {
            for figure in [0.5, 0.75, 0.9, 1.0] {
                let analysis = rated(1.0, Some(figure));
                let waiting = name_confidence(&analysis, doubts, true);
                assert!(waiting <= WAITING_CEILING, "{doubts} {figure}");
                assert!(waiting <= name_confidence(&analysis, doubts, false));
                assert!(
                    name_confidence(&analysis, doubts + 1, false)
                        < name_confidence(&analysis, doubts, false)
                );
            }
        }
        // An absurd count of reasons is no overflow.
        assert!(name_confidence(&sure, usize::MAX, true) >= 0.0);
    }

    /// The level is stored as a lowercase word, strict is the default, and
    /// a stored value this build does not know reads as strict.
    #[test]
    fn the_level_is_stored_as_a_word_and_an_unknown_one_reads_as_strict() {
        assert_eq!(ReviewLevel::default(), ReviewLevel::Strict);
        assert!(ReviewLevel::Strict.is_strict());
        assert!(!ReviewLevel::Balanced.is_strict());
        for level in ReviewLevel::ALL {
            let written = serde_json::to_value(level).unwrap();
            assert_eq!(written, serde_json::json!(level.as_str()));
            assert_eq!(
                serde_json::from_value::<ReviewLevel>(written).unwrap(),
                level
            );
            assert_eq!(ReviewLevel::parse(level.as_str()), Some(level));
        }
        assert_eq!(ReviewLevel::Balanced.as_str(), "balanced");
        assert_eq!(ReviewLevel::Strict.as_str(), "strict");
        for unknown in [
            serde_json::json!("relaxed"),
            serde_json::json!("Balanced"),
            serde_json::json!(" balanced"),
            serde_json::json!(""),
            serde_json::json!(2),
            serde_json::json!(true),
            serde_json::json!(null),
            serde_json::json!(["balanced"]),
            serde_json::json!({"level": "balanced"}),
        ] {
            assert_eq!(
                serde_json::from_value::<ReviewLevel>(unknown.clone()).unwrap(),
                ReviewLevel::Strict,
                "{unknown}"
            );
        }
        assert_eq!(ReviewLevel::parse("Balanced"), None);
    }
}
