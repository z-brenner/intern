//! The words a document gives, and the words a learned spelling adds.
//!
//! A learned spelling is checked by a person approving a name it wrote, on
//! a document new to Intern (see [`crate::pipeline::LearnedRule::confirmed`]).
//! Such an approval shows the spelling is right only where the document does
//! not give, in its own words, what the spelling adds. "John Smith, Jane
//! Doe", learned for "John Smith" from two agreements a reviewer named that
//! way, is the right name for a third agreement between John Smith, Jane Doe
//! and Acme - and approving it there says nothing about the fourth, which
//! names John Smith alone. So the queue keeps, with each reading, which words
//! the document's text gives ([`DocumentWords`]), and reads an approval
//! against them long after the text itself is gone.

use std::collections::HashSet;

use intern_engine::{DocumentSource, HouseRule, engine::barely_readable};
use serde::{Deserialize, Serialize};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// The words that end a company's name, as house style compares spellings
/// ([`HouseRule::key`]): a note's start just in front of one is a word of
/// the name ("Swiss Re Ltd", "Munich Re Group", "Munich Re Capital
/// Limited"), not a note; and a spelling that adds one of the legal forms
/// among them adds no one (see [`is_legal_form`]).
pub(crate) const COMPANY_FORMS: [&str; 19] = [
    "limited",
    "company",
    "incorporated",
    "ltd",
    "inc",
    "llc",
    "llp",
    "lp",
    "plc",
    "group",
    "holdings",
    "corporation",
    "corp",
    "co",
    "ag",
    "se",
    "sa",
    "nv",
    "gmbh",
];

/// Words that join the words of a name and name no one: every document
/// gives them, so they cannot tell a document that gives a spelling from
/// one that does not, and a spelling that adds only these ("Bank of
/// America" for "Bank America") adds no one.
const JOINING_WORDS: [&str; 11] = [
    "a", "an", "the", "of", "and", "for", "at", "in", "on", "to", "by",
];

/// Bits a word takes in a [`DocumentWords`], for about one word in a
/// hundred and twenty answered as given when it is not.
const BITS_PER_WORD: usize = 10;
/// How many bits each word sets.
const HASHES: u64 = 7;
/// The smallest and the largest filter kept, in bits. A document with more
/// words than the largest holds well is answered as giving more words than
/// it does, which counts no check where one could have counted: the next
/// document waits again.
const MIN_BITS: usize = 1 << 10;
const MAX_BITS: usize = 1 << 17;

/// Which words a document's text gives, as house style compares spellings
/// (see [`HouseRule::key`]) once accents are taken out (see [`plain`]):
/// each run of characters between spaces with its punctuation taken out
/// ("Doe," is "doe", "O'Brien" is "obrien", "José" is "jose"), and each run
/// of letters and digits in it ("Smith-Jones" gives "smith" and "jones" as
/// well as "smithjones").
///
/// Kept as a Bloom filter, about a byte and a quarter a word: it can answer
/// that a word is given when it is not - and then counts no check, which
/// leaves the next document waiting - but never that a word the text gives
/// is missing. Stored with the reading, as sixteen hex digits per 64 bits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(from = "String", into = "String")]
pub struct DocumentWords {
    /// Empty for a stored value that could not be read, which gives every
    /// word.
    bits: Vec<u64>,
}

impl DocumentWords {
    /// The words of everything the reader read from `source`. `None` when
    /// it read no words, when a page could not be read as text at all
    /// ([`barely_readable`]), or when OCR was unsure of what it read
    /// (`LOW_OCR_CONFIDENCE`), so that what the document gives is not
    /// known. A scan whose page came with an image of it is no less known
    /// than its text: the image is not shown to the model, so the text is
    /// all the reading had, as for every other page OCR read.
    pub fn of(source: &DocumentSource) -> Option<Self> {
        if barely_readable(source)
            || source
                .parser_warnings
                .iter()
                .any(|warning| warning.code == "LOW_OCR_CONFIDENCE")
        {
            return None;
        }
        let mut words = HashSet::new();
        for page in &source.pages {
            for written in page.text.split_whitespace() {
                words.extend(word_keys(written));
            }
        }
        if words.is_empty() {
            return None;
        }
        let bits = (words.len() * BITS_PER_WORD)
            .next_power_of_two()
            .clamp(MIN_BITS, MAX_BITS);
        let mut kept = Self {
            bits: vec![0; bits / 64],
        };
        for word in &words {
            for bit in kept.positions(word) {
                kept.bits[bit / 64] |= 1 << (bit % 64);
            }
        }
        Some(kept)
    }

    /// Whether the document gives `word`, a word as house style compares
    /// spellings. Never false for a word it gives.
    pub fn gives(&self, word: &str) -> bool {
        self.bits.is_empty()
            || self
                .positions(word)
                .all(|bit| self.bits[bit / 64] & (1 << (bit % 64)) != 0)
    }

    /// Sixteen hex digits that are the same for two documents giving the
    /// same words, whatever their order, and almost never for two that do
    /// not: a teacher of a learned spelling too short for a text
    /// fingerprint, scanned again, is known by it (see
    /// [`crate::pipeline::LearnedRule::confirmed`]).
    pub fn digest(&self) -> String {
        let bytes = self
            .bits
            .iter()
            .flat_map(|bits| bits.to_le_bytes())
            .collect::<Vec<_>>();
        format!("{:016x}", fnv1a(&bytes))
    }

    fn positions(&self, word: &str) -> impl Iterator<Item = usize> + use<> {
        let size = (self.bits.len() * 64) as u64;
        let first = fnv1a(word.as_bytes());
        let step = mix(first) | 1;
        (0..HASHES).map(move |index| (first.wrapping_add(index.wrapping_mul(step)) % size) as usize)
    }
}

impl From<DocumentWords> for String {
    fn from(words: DocumentWords) -> Self {
        words
            .bits
            .iter()
            .map(|bits| format!("{bits:016x}"))
            .collect()
    }
}

impl From<String> for DocumentWords {
    /// A value that is not one this build writes gives every word, so it
    /// counts no check.
    fn from(stored: String) -> Self {
        let bits = (stored.len() % 16 == 0)
            .then(|| {
                (0..stored.len())
                    .step_by(16)
                    .map(|at| {
                        stored
                            .get(at..at + 16)
                            .and_then(|digits| u64::from_str_radix(digits, 16).ok())
                    })
                    .collect::<Option<Vec<_>>>()
            })
            .flatten()
            .filter(|bits| bits.len().is_power_of_two() && bits.len() * 64 >= MIN_BITS)
            .unwrap_or_default();
        Self { bits }
    }
}

/// The words a learned spelling writes that the document's own word does
/// not hold, when it adds any: each way of reading the words of
/// `rule.to` - as written between spaces, or split at punctuation as well -
/// less the words of `rule.from` read either way, the legal forms
/// ([`is_legal_form`]) and the words that only join others
/// ([`JOINING_WORDS`]). A document gives what the rule adds when it gives
/// any of their words (see [`gives`]).
///
/// `None` - nothing a document could give in its own words - where either
/// reading leaves no word: a short form ("Acme" for "Acme Corporation"), a
/// change of case, accents or punctuation, a legal form added or changed ("Acme
/// Corporation" for "Acme", "ACME Corp" for "ACME CORPORATION"); and for an
/// acronym of the document's word ("DEQ" for "Department of Environmental
/// Quality"), which adds letters, not a party, and whose letters a document
/// gives beside the name as often as not. Not for the words an acronym
/// stands for ("American Broadcasting Company" for "ABC"): they are words
/// a document gives or does not, and one that spells them out itself says
/// nothing about the next "ABC", which may be ABC Supply.
pub fn added(rule: &HouseRule) -> Option<Vec<Vec<String>>> {
    if is_acronym(&rule.from, &rule.to) {
        return None;
    }
    let own = words_of(&rule.from)
        .into_iter()
        .chain(pieces_of(&rule.from))
        .collect::<HashSet<_>>();
    let adds = |words: Vec<String>| {
        words
            .into_iter()
            .filter(|word| {
                !own.contains(word)
                    && !is_legal_form(word)
                    && !JOINING_WORDS.contains(&word.as_str())
            })
            .collect::<Vec<_>>()
    };
    let readings = [adds(words_of(&rule.to)), adds(pieces_of(&rule.to))];
    (!readings.iter().any(Vec::is_empty)).then(|| readings.into())
}

/// Whether `word` is a company's legal form ([`COMPANY_FORMS`]), which
/// names no one the company's name does not: not "Group" or "Holdings",
/// which end a company's name as surely but name another company of the
/// same family - "Acme Holdings", the parent of Acme, is not Acme.
fn is_legal_form(word: &str) -> bool {
    COMPANY_FORMS.contains(&word) && !matches!(word, "group" | "holdings")
}

/// Whether a document gives what a rule adds (see [`added`]), or any part
/// of it: a word of it in `text`, the words its text gives, or among the
/// words of `named`, the names its reading gives (its parties, its type).
/// A part is enough. A document that writes "J. Doe" or "Ms. Doe" can make
/// "John Smith, Jane Doe" read as its own words to the person approving
/// it, as surely as one that writes "Jane Doe" or "Doe, Jane"; and asking
/// for the whole as a run of words is more than the words kept can answer.
/// The price is a spelling that adds two words or more, one of which the
/// document gives on its own, waiting for another document's approval.
/// Without the words of its text - a reading stored before they were kept,
/// or a document read from a page image - it is not known, and taken to
/// give them: in doubt, no check.
pub fn gives(added: &[Vec<String>], text: Option<&DocumentWords>, named: &[&str]) -> bool {
    let Some(text) = text else {
        return true;
    };
    let named = named
        .iter()
        .flat_map(|name| words_of(name).into_iter().chain(pieces_of(name)))
        .collect::<HashSet<_>>();
    added
        .iter()
        .flatten()
        .any(|word| named.contains(word) || text.gives(word))
}

/// Whether `short` is an acronym of `long`: a single word, but for any
/// company form ("IBM Corp"), of two letters or more, made of the first
/// letters of `long`'s words - all of them, or leaving out the words that
/// only join others or the company forms ("DEQ" for "Department of
/// Environmental Quality", "IBM" for "International Business Machines
/// Corporation"), each reading of the words (see [`added`]) - or of its
/// last words, leaving out a prefix in front of them ("EPA" for "U.S.
/// Environmental Protection Agency", "DEQ" for "Oregon Department of
/// Environmental Quality"). The letters run to the last word: "DOE" is not
/// "Department of Environmental Quality", nor its "Department of".
fn is_acronym(long: &str, short: &str) -> bool {
    let short = words_of(short)
        .into_iter()
        .filter(|word| !COMPANY_FORMS.contains(&word.as_str()))
        .collect::<Vec<_>>();
    let [short] = short.as_slice() else {
        return false;
    };
    if short.chars().count() < 2 {
        return false;
    }
    [words_of(long), pieces_of(long)].iter().any(|words| {
        words.len() >= 2
            && [(false, false), (true, false), (false, true), (true, true)]
                .iter()
                .any(|&(joining, forms)| {
                    let initials = words
                        .iter()
                        .filter(|word| !joining || !JOINING_WORDS.contains(&word.as_str()))
                        .filter(|word| !forms || !COMPANY_FORMS.contains(&word.as_str()))
                        .filter_map(|word| word.chars().next())
                        .collect::<String>();
                    initials.chars().count() >= 2 && initials.ends_with(short.as_str())
                })
    })
}

/// `value` as a person reads it, whatever code points spell it: without
/// its accents, ligatures and wide or compatibility forms. "José" typed
/// one way, "Jose" with a combining accent as macOS and some PDFs write
/// it, and "Jose" are one word to the person approving a name, so they
/// are one word here: a document gives the word however it writes the
/// accent, and a spelling that changes accents alone ("Société Générale"
/// for "Societe Generale") adds no word, as one that changes case adds
/// none.
fn plain(value: &str) -> String {
    value
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .collect()
}

/// The words of `value` as written between spaces, as house style compares
/// spellings, read [`plain`]: "Smith-Jones, Inc." is "smithjones" and "inc".
fn words_of(value: &str) -> Vec<String> {
    HouseRule::key(&plain(value))
        .split(' ')
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The words of `value` split at punctuation as well, read [`plain`]:
/// "Smith-Jones, Inc." is "smith", "jones" and "inc".
fn pieces_of(value: &str) -> Vec<String> {
    plain(value)
        .split(|character: char| !character.is_alphanumeric())
        .map(HouseRule::key)
        .filter(|word| !word.is_empty())
        .collect()
}

/// The words a run of characters between spaces gives, read both ways and
/// [`plain`].
fn word_keys(written: &str) -> Vec<String> {
    let written = plain(written);
    std::iter::once(HouseRule::key(&written))
        .chain(
            written
                .split(|character: char| !character.is_alphanumeric())
                .map(HouseRule::key),
        )
        .filter(|word| !word.is_empty())
        .collect()
}

/// FNV-1a, spelled out so a stored filter reads the same in every build.
fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
    })
}

/// A second, independent hash of a word from its first (splitmix64's
/// finalizer), for the filter's double hashing.
fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use intern_engine::{
        DocumentSource, HouseRule, PageImage, PageOrigin, ParserWarning, RuleKind, SourcePage,
    };

    use super::{DocumentWords, added, gives};

    fn text(text: &str) -> DocumentWords {
        DocumentWords::of(&DocumentSource::from_pages(vec![SourcePage::new(
            1,
            text,
            PageOrigin::Native,
        )]))
        .unwrap()
    }

    fn party(from: &str, to: &str) -> HouseRule {
        HouseRule::new(RuleKind::Party, from, to)
    }

    /// Every word of the text is given, however it is written - capitals,
    /// punctuation around it, a line break before it, joined to another by
    /// a hyphen - and a word kept through a stored value is still given.
    #[test]
    fn every_word_the_text_gives_is_given() {
        let words =
            text("AGREEMENT between Doe, Jane\nand SMITH-JONES Holdings; signed by O'Brien (DEQ).");
        for word in [
            "agreement",
            "between",
            "doe",
            "jane",
            "smithjones",
            "smith",
            "jones",
            "holdings",
            "obrien",
            "o",
            "brien",
            "deq",
        ] {
            assert!(words.gives(word), "{word}");
        }
        let stored: String = words.clone().into();
        let again = DocumentWords::from(stored);
        assert_eq!(again, words);
        assert_eq!(again.digest(), words.digest());
        assert_eq!(
            text("signed by O'Brien (DEQ). AGREEMENT between Doe, Jane\nand SMITH-JONES Holdings;")
                .digest(),
            words.digest(),
            "the same words in another order"
        );
        assert_ne!(text("AGREEMENT between Doe, Jane").digest(), words.digest());
        assert!(again.gives("jane"));
        assert!(!again.gives("john"), "a word the text does not give");
    }

    /// A long text keeps every word too, past the largest filter; a stored
    /// value this build did not write gives every word, so it counts no
    /// check; a page read as an image gives no words to go by.
    #[test]
    fn a_long_text_keeps_every_word_and_an_unreadable_one_gives_all() {
        let long = (0..40_000)
            .map(|index| format!("w{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let words = text(&long);
        assert!((0..40_000).all(|index| words.gives(&format!("w{index}"))));
        assert!(DocumentWords::from("not hex".to_owned()).gives("anything"));
        assert!(DocumentWords::from(String::new()).gives("anything"));
        let mut image = DocumentSource::from_pages(Vec::new());
        image.page_image = Some(PageImage {
            page_number: 1,
            media_type: "image/png".into(),
            bytes: Vec::new(),
        });
        assert_eq!(DocumentWords::of(&image), None);
    }

    /// What each kind of spelling adds: nothing for a short form, a change
    /// of case, accents or punctuation, a legal form, or an acronym of the
    /// document's word, a prefix in front of it left out or not; the new
    /// words, read both ways, for a party or a name added, another company
    /// of the same family, or the words an acronym stands for.
    #[test]
    fn a_spelling_adds_only_words_of_its_own() {
        for (from, to) in [
            ("Acme Corporation", "Acme"),
            ("ACME CORP", "Acme Corp"),
            ("ACME CORPORATION", "ACME Corp"),
            ("Acme", "Acme Corporation"),
            ("Acme", "Acme, Inc."),
            ("Smith Jones", "Smith-Jones"),
            ("OBrien", "O'Brien"),
            ("Bank America", "Bank of America"),
            ("Department of Environmental Quality", "DEQ"),
            ("Department of Environmental Quality", "D.E.Q."),
            ("International Business Machines Corporation", "IBM Corp"),
            ("U.S. Environmental Protection Agency", "EPA"),
            ("Oregon Department of Environmental Quality", "DEQ"),
            ("Societe Generale", "Soci\u{e9}t\u{e9} G\u{e9}n\u{e9}rale"),
        ] {
            assert_eq!(added(&party(from, to)), None, "{from} -> {to}");
        }
        let words = |readings: Option<Vec<Vec<String>>>| {
            readings.map(|readings| readings.into_iter().flatten().collect::<Vec<_>>())
        };
        assert_eq!(
            words(added(&party("John Smith", "John Smith, Jane Doe"))),
            Some(vec![
                "jane".into(),
                "doe".into(),
                "jane".into(),
                "doe".into()
            ])
        );
        assert_eq!(
            words(added(&party("Smith", "John Smith"))),
            Some(vec!["john".into(), "john".into()])
        );
        assert_eq!(
            words(added(&party("Acme", "Acme Holdings Trust"))),
            Some(vec![
                "holdings".into(),
                "trust".into(),
                "holdings".into(),
                "trust".into()
            ])
        );
        assert_eq!(
            words(added(&party("Acme", "Acme Holdings"))),
            Some(vec!["holdings".into(), "holdings".into()]),
            "the parent of Acme is not Acme"
        );
        assert_eq!(
            words(added(&party("Acme", "Acme Group, Inc."))),
            Some(vec!["group".into(), "group".into()])
        );
        assert_eq!(
            words(added(&party("ABC", "American Broadcasting Company"))),
            Some(vec![
                "american".into(),
                "broadcasting".into(),
                "american".into(),
                "broadcasting".into()
            ]),
            "the words an acronym stands for"
        );
        assert!(
            added(&party("Department of Environmental Quality", "DOE")).is_some(),
            "not its initials"
        );
        assert!(
            added(&party("Department of Environmental Quality", "DE")).is_some(),
            "nor the initials of its first words"
        );
    }

    /// A word is given however its accents are written: "José" in the text
    /// gives "Jose" typed without the accent, and "Jose" with a combining
    /// accent - as macOS and some PDFs write it - gives "José" typed with
    /// one; a ligature gives the letters it joins. Each of them counted no
    /// word given, so an approval of a document giving the name was taken
    /// as the check of a spelling adding it.
    #[test]
    fn a_word_is_given_whatever_code_points_write_its_accents() {
        let jose = added(&party("Smith", "Jose Smith")).unwrap();
        let accented = added(&party("Smith", "Jos\u{e9} Smith")).unwrap();
        let composed = text("Agreement between Jos\u{e9} Smith and Acme Corporation.");
        let decomposed = text("Agreement between Jose\u{301} Smith and Acme Corporation.");
        for (adds, document) in [
            (&jose, &composed),
            (&jose, &decomposed),
            (&accented, &composed),
            (&accented, &decomposed),
        ] {
            assert!(gives(adds, Some(document), &[]), "{adds:?}");
        }
        assert!(gives(
            &jose,
            Some(&text("Agreement.")),
            &["Jos\u{e9} Smith"]
        ));
        assert!(gives(
            &added(&party("Acme", "Acme Fire Insurance")).unwrap(),
            Some(&text("Agreement with Acme \u{fb01}re Insurance.")),
            &[]
        ));
        assert!(!gives(
            &accented,
            Some(&text("Agreement between Mr. Smith and Acme Corporation.")),
            &[]
        ));
    }

    /// A scanned page that came with an image of it gives the words its OCR
    /// read: the image is not shown to the model, so the text is everything
    /// the reading had. Not a page that could not be read as text, nor a
    /// reading OCR was unsure of. Every scan's first page used to give no
    /// words, so no approval of a scan ever checked a spelling adding one.
    #[test]
    fn a_scan_gives_the_words_its_ocr_read_with_confidence() {
        let scan = |text: &str, unsure: bool| {
            let mut page = SourcePage::new(1, text, PageOrigin::Ocr);
            page.ocr_confidence = Some(if unsure { 41 } else { 91 });
            let mut source = DocumentSource::from_pages(vec![page]);
            source.page_image = Some(PageImage {
                page_number: 1,
                media_type: "image/png".into(),
                bytes: vec![1, 2, 3],
            });
            if unsure {
                source
                    .parser_warnings
                    .push(ParserWarning::new("LOW_OCR_CONFIDENCE", true));
            }
            source
        };
        let read = "Employment Agreement signed May 3, 2024. This agreement is made by Mr. \
                    Smith and Acme Corporation. The employee shall serve as warehouse \
                    supervisor and report to the operations manager. Either party may end \
                    this agreement with thirty days written notice.";
        let words = DocumentWords::of(&scan(read, false)).unwrap();
        assert!(words.gives("smith") && words.gives("warehouse"));
        assert!(!words.gives("john"));
        assert_eq!(DocumentWords::of(&scan(read, true)), None, "OCR unsure");
        assert_eq!(
            DocumentWords::of(&scan("l1 ll I Smith", false)),
            None,
            "not read as text"
        );
    }

    /// A document gives what a spelling adds when its text, or a name its
    /// reading gives, holds any added word - in any order ("Doe, Jane"), or
    /// part of it ("J. Doe") - and not when it holds none; without its
    /// text, it is taken to.
    #[test]
    fn a_document_gives_what_a_spelling_adds_with_any_word_of_it() {
        let joined = added(&party("John Smith", "John Smith, Jane Doe")).unwrap();
        let first = added(&party("Smith", "John Smith")).unwrap();
        let both = text("Agreement between John Smith, Jane Doe and Acme Corporation.");
        let inverted = text("Agreement between Smith, John and Doe, Jane.");
        let initial = text("Agreement between John Smith, J. Doe and Acme Corporation.");
        let alone = text("Agreement between Mr. Smith and Acme Corporation.");
        assert!(gives(&joined, Some(&both), &[]));
        assert!(gives(&joined, Some(&inverted), &[]));
        assert!(gives(&joined, Some(&initial), &[]), "part of it");
        assert!(!gives(&joined, Some(&alone), &[]));
        assert!(
            gives(&joined, Some(&alone), &["Jane Doe"]),
            "a party the reading gives"
        );
        assert!(!gives(&first, Some(&alone), &["Smith", "Acme Corporation"]));
        assert!(gives(&first, Some(&both), &[]));
        assert!(gives(&first, None, &[]), "in doubt");
    }
}
