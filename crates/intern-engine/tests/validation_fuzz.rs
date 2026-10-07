//! Seeded random text around real dates, through validation and naming.
//!
//! Validation and naming slice document text by byte offset in many places,
//! and one slice that landed inside "é" once panicked on every French
//! invoice - inside the model thread, where it paused the whole queue. This
//! surrounds a real date, in each spelling documents use, with random text
//! drawn from what extraction actually hands over - accents, symbols, CJK,
//! Hangul, emoji, combining marks, direction marks, ligatures - and the
//! wording the date checks key on. Of every case it asks two things: nothing
//! panics, and any date that comes out is one the document states.
//!
//! Deterministic: each case has its own seed, printed when it fails, so a
//! failure reproduces on its own.

use std::panic::catch_unwind;

use intern_engine::distill::{DigestBudget, distill, source_from_text};
use intern_engine::domain::{AnalysisTelemetry, DateRole, Evidence, ModelProposal, PartyRelation};
use intern_engine::engine::finish;
use intern_engine::evidence::digest_contains_date;
use intern_engine::validate::validate;

const CASES: u64 = 3_000;

/// xorshift64 - no dependency, and plenty to scatter text with.
struct Rng(u64);

impl Rng {
    fn seeded(case: u64) -> Self {
        let seed = (case + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % bound as u64).expect("below a usize bound")
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }
}

/// Plain words, and the wording the date, reference, deadline, party, and
/// description checks key on.
const WORDS: &[&str] = &[
    "the",
    "this",
    "of",
    "to",
    "and",
    "&",
    "between",
    "by",
    "under",
    "as",
    "Agreement",
    "Contract",
    "Order",
    "Amendment",
    "Memorandum",
    "Contractor",
    "border",
    "dated",
    "updated",
    "effective",
    "commencing",
    "Invoice",
    "Date:",
    "Due",
    "Date",
    "Payment",
    "due",
    "payable",
    "Expires",
    "renewal",
    "Statement",
    "Notice",
    "Termination",
    "issued",
    "pursuant",
    "amended",
    "amending",
    "Section",
    "9.2",
    "Acme",
    "Corporation",
    "Contoso",
    "Inc.",
    "P.C.",
    "U.K.",
    "Ltd.",
    "No.",
    "Jan.",
    "Sept.",
    "$1,248.00",
    "04/30/2025",
    "12/1/2026",
    "2025",
    "(this",
    "\"Agreement\")",
    ".",
    ",",
    ";",
    ":",
    "-",
    "|",
    "**Date:**",
    "#",
];

/// Characters that are more than one byte in UTF-8, or that NFKC and case
/// folding change the length of.
const SYMBOLS: &[&str] = &[
    "é", "É", "ü", "ß", "ẞ", "İ", "§", "•", "°", "☐", "€", "£", "½", "¼", "²", "文", "書", "日",
    "付", "한", "국", "어", "😀", "📄", "👍🏽", "e\u{301}", "\u{308}", "\u{200E}", "\u{FB01}",
    "\u{FB02}", "\u{2014}", "\u{2019}", "\u{00A0}", "\u{2007}", "\u{3000}",
];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn ordinal(day: u32) -> &'static str {
    match day {
        11..=13 => "th",
        _ if day % 10 == 1 => "st",
        _ if day % 10 == 2 => "nd",
        _ if day % 10 == 3 => "rd",
        _ => "th",
    }
}

/// A real calendar date, as ISO and as one of ten spellings documents use.
fn random_date(rng: &mut Rng) -> (String, String) {
    let year = 1990 + u32::try_from(rng.below(46)).expect("small");
    let month = 1 + rng.below(12);
    let day = 1 + u32::try_from(rng.below(28)).expect("small");
    let name = MONTHS[month - 1];
    let iso = format!("{year:04}-{month:02}-{day:02}");
    let spelled = match rng.below(10) {
        0 => format!("{name} {day}, {year}"),
        1 => format!("{day} {name} {year}"),
        2 => format!("{}. {day}, {year}", &name[..3]),
        3 => format!("the {day}{} day of {name}, {year}", ordinal(day)),
        4 => format!("{month:02}/{day:02}/{year}"),
        5 => format!("{day:02}/{month:02}/{year}"),
        6 => format!("{month}/{day}/{:02}", year % 100),
        7 => iso.clone(),
        8 => format!("{day:02}.{month:02}.{year}"),
        _ => format!("{} {day} {year}", name.to_uppercase()),
    };
    (iso, spelled)
}

fn random_line(rng: &mut Rng) -> String {
    let mut line = String::new();
    for _ in 0..1 + rng.below(14) {
        if rng.chance(55) {
            line.push_str(rng.pick(WORDS));
        } else {
            for _ in 0..1 + rng.below(4) {
                line.push_str(rng.pick(SYMBOLS));
            }
        }
        if rng.chance(85) {
            line.push(' ');
        }
    }
    line
}

/// A random character boundary of `text`, ends included.
fn boundary(rng: &mut Rng, text: &str) -> usize {
    let boundaries: Vec<usize> = text
        .char_indices()
        .map(|(index, _)| index)
        .chain([text.len()])
        .collect();
    rng.pick(&boundaries)
}

/// A random stretch of `text`, cut on character boundaries.
fn substring(rng: &mut Rng, text: &str) -> String {
    let first = boundary(rng, text);
    let second = boundary(rng, text);
    text[first.min(second)..first.max(second)].to_owned()
}

/// Builds and runs one case; `true` when a date came out of it.
fn run_case(case: u64) -> bool {
    let mut rng = Rng::seeded(case);
    let (iso, spelled) = random_date(&mut rng);
    let mut lines: Vec<String> = (0..1 + rng.below(6))
        .map(|_| random_line(&mut rng))
        .collect();
    // The date goes in once or twice, at any character boundary - glued to
    // whatever is there as often as set apart by spaces.
    for _ in 0..1 + rng.below(2) {
        let index = rng.below(lines.len());
        let line = &mut lines[index];
        let at = boundary(&mut rng, line);
        let inserted = if rng.chance(70) {
            format!(" {spelled} ")
        } else {
            spelled.clone()
        };
        line.insert_str(at, &inserted);
    }
    let text = lines.join("\n");

    let digest = distill(&source_from_text(&text), DigestBudget::default());
    let parties = (0..rng.below(3))
        .map(|_| substring(&mut rng, &text))
        .collect::<Vec<_>>();
    let document_type = match rng.below(5) {
        0 => None,
        1 => Some("Invoice".to_owned()),
        2 => Some("Consulting Agreement".to_owned()),
        3 => Some("Notice of Termination".to_owned()),
        _ => Some(substring(&mut rng, &text)),
    };
    let description = if rng.chance(50) {
        format!(
            "Invoice dated {spelled} from {}.",
            substring(&mut rng, &text)
        )
    } else {
        substring(&mut rng, &text)
    };
    let candidate = ModelProposal {
        document_type,
        document_date: Some(iso),
        date_role: Some(rng.pick(&DateRole::ALL)),
        parties,
        party_relation: rng.pick(&PartyRelation::ALL),
        description,
        confidence: 0.9,
        needs_review: rng.chance(10),
        evidence: Evidence {
            date: Some(substring(&mut rng, &text)),
            document_type: Some(substring(&mut rng, &text)),
            parties: vec![substring(&mut rng, &text)],
        },
        facts: None,
    };

    let outcome = validate(candidate, &digest);
    let analysis = finish(
        outcome,
        &digest,
        "pdf",
        &["existing.pdf"],
        AnalysisTelemetry::default(),
    );
    match analysis.proposal.document_date.as_deref() {
        Some(date) => {
            assert!(
                digest_contains_date(&digest, date),
                "case {case}: accepted {date}, which the document does not state"
            );
            true
        }
        None => false,
    }
}

#[test]
fn random_unicode_around_dates_never_panics_validation_or_naming() {
    let mut accepted = 0;
    for case in 0..CASES {
        match catch_unwind(|| run_case(case)) {
            Ok(true) => accepted += 1,
            Ok(false) => {}
            Err(_) => panic!("case {case} panicked; rerun it alone with Rng::seeded({case})"),
        }
    }
    // The cases must reach the date paths, not only the rejection of a date
    // the text mangled: most dates sit apart from what surrounds them.
    assert!(
        accepted > CASES / 2,
        "only {accepted} of {CASES} cases accepted a date"
    );
}
