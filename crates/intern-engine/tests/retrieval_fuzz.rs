//! Seeded random pages through the evidence index and retrieval.
//!
//! Index and retrieval run before the analysis guard and slice text by
//! byte offset - dates, labels, sentence chunks, organisation names - so
//! they get the treatment validation gets in `validation_fuzz.rs`: random
//! text from what extraction hands over (accents, symbols, CJK, Hangul,
//! emoji, combining marks, direction marks, ligatures), the wording the
//! cues key on, real dates in every spelling, and the structure the
//! segmenter reads (headings, `Key: value` lines, `|` tables, list
//! markers, blank lines), over one to ten pages. Of every case it asks:
//! nothing panics; every unit is text its page holds; ids are unique;
//! every date a unit carries is one the evidence check finds where it
//! stands; the context holds only units of the document, none running,
//! within budget unless the document went whole; and the same input gives
//! the same context.
//!
//! Deterministic: each case has its own seed, printed when it fails.

use std::collections::BTreeSet;
use std::panic::catch_unwind;

use intern_engine::domain::{DocumentSource, PageOrigin, SourcePage};
use intern_engine::evidence::{date_match_positions, normalize};
use intern_engine::index::EvidenceIndex;
use intern_engine::retrieve::{
    IdStyle, RetrievalConfig, Strategy, Tier, prepare_evidence, retrieve,
};

const CASES: u64 = 300;

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

/// A line the segmenter reads as structure: a heading, a labelled value, a
/// table row or separator, a list item, or plain text.
fn structured_line(rng: &mut Rng) -> String {
    let line = random_line(rng);
    match rng.below(8) {
        0 => line.to_uppercase(),
        1 => format!(
            "{}: {line}",
            rng.pick(&["Invoice Date", "Bill To", "Tenant", "Policy No.", "Total"])
        ),
        2 => format!("| {} | {} |", random_line(rng), line),
        3 => "| --- | --- |".to_owned(),
        4 => format!("{} {line}", rng.pick(&["-", "•", "(a)", "(iv)", "3)"])),
        5 => format!("# {line}"),
        6 => String::new(),
        _ => line,
    }
}

fn run_case(case: u64) -> usize {
    let mut rng = Rng::seeded(case);
    let pages = (0..1 + rng.below(10))
        .map(|number| {
            let mut lines: Vec<String> = (0..1 + rng.below(30))
                .map(|_| structured_line(&mut rng))
                .collect();
            for _ in 0..rng.below(3) {
                let (_, spelled) = random_date(&mut rng);
                let index = rng.below(lines.len());
                let line = &mut lines[index];
                let at = boundary(&mut rng, line);
                line.insert_str(at, &format!(" {spelled} "));
            }
            if rng.chance(30) {
                lines.push(format!("Page {} of 12", number + 1));
            }
            let origin = if rng.chance(20) {
                PageOrigin::Ocr
            } else {
                PageOrigin::Native
            };
            let mut page = SourcePage::new(number + 1, lines.join("\n"), origin);
            page.ocr_confidence = (origin == PageOrigin::Ocr).then(|| rng.below(101) as u32);
            page
        })
        .collect::<Vec<_>>();
    let source = DocumentSource::from_pages(pages);
    let config = RetrievalConfig {
        max_unit_chars: [60, 250, 400][rng.below(3)],
        whole_document_tokens: [0, 200, 1_800][rng.below(3)],
        strategy: [Strategy::Flat, Strategy::Hierarchical, Strategy::Auto][rng.below(3)],
        id_style: [IdStyle::Stable, IdStyle::Ordinal][rng.below(2)],
        units_per_date: rng.below(3) as u8,
        ..RetrievalConfig::default()
    };
    let scale = [25, 50, 100][rng.below(3)];

    let (index, context) = prepare_evidence(&source, &config, scale)
        .unwrap_or_else(|_| panic!("case {case}: index or retrieval panicked"));
    let ids = index
        .units()
        .iter()
        .map(|unit| unit.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), index.units().len(), "case {case}: duplicate ids");
    for unit in index.units() {
        let page = source
            .pages
            .iter()
            .find(|page| page.page_number == unit.page)
            .expect("a unit's page");
        assert!(
            normalize(&page.text).contains(&unit.normalized),
            "case {case}: {} is not text of page {}",
            unit.id,
            unit.page
        );
        for mention in &unit.features.dates {
            assert!(
                !date_match_positions(&mention.iso, &unit.normalized).is_empty()
                    || unit.text.lines().count() > 1,
                "case {case}: {} has {} it does not state",
                unit.id,
                mention.iso
            );
        }
    }
    let mut previous = None;
    for ordinal in &context.units {
        let unit = &index.units()[*ordinal as usize];
        assert!(!unit.running, "case {case}: a running line in the context");
        assert!(
            previous < Some(*ordinal),
            "case {case}: out of document order"
        );
        previous = Some(*ordinal);
    }
    assert_eq!(context.handles.len(), context.units.len());
    if context.tier != Tier::Whole {
        let ceiling = config.budgets.total() as usize * config.dense_pct as usize / 100;
        // A field may take one unit over its budget when nothing smaller
        // fits; that unit is at most four budgets.
        assert!(
            context.estimated_tokens <= ceiling * 4,
            "case {case}: {} tokens",
            context.estimated_tokens
        );
    }
    assert_eq!(
        retrieve(&index, &config, scale),
        context,
        "case {case}: retrieval is not deterministic"
    );
    if case % 5 == 0 {
        assert_eq!(
            EvidenceIndex::build_with(&source, config.index_options()),
            index,
            "case {case}: the index is not deterministic"
        );
    }
    context.units.len()
}

#[test]
fn random_pages_never_panic_the_index_or_retrieval() {
    let mut retrieved = 0;
    for case in 0..CASES {
        match catch_unwind(|| run_case(case)) {
            Ok(units) => retrieved += usize::from(units > 0),
            Err(_) => panic!("case {case} failed; rerun it alone with Rng::seeded({case})"),
        }
    }
    assert!(
        retrieved > CASES as usize / 2,
        "only {retrieved} cases retrieved anything"
    );
}
