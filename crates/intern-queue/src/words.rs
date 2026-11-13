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

use intern_engine::{DocumentSource, HouseRule, PageOrigin, engine::barely_readable};
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

/// The characters a word broken at the end of a line ends with, before
/// the rest of it on the next: hyphens and dashes, the soft hyphen, and the
/// "not" sign some PDFs and OCR engines write for a hyphen there.
const LINE_BREAK_HYPHENS: [char; 13] = [
    '-', '\u{ad}', '\u{ac}', '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}',
    '\u{2212}', '\u{fe58}', '\u{fe63}', '\u{ff0d}', '\u{1806}',
];

/// The most letters of a name spaced out as initials ("J. P.", "U S A")
/// read as one word.
const MAX_INITIALS: usize = 6;

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
/// (see [`HouseRule::key`]) once they are read [`spellings`]: each run of
/// characters between spaces with its punctuation taken out ("Doe," is
/// "doe", "O'Brien" is "obrien", "José" is "jose", "Müller" is "muller"
/// and "mueller"), each run of letters and digits in it ("Smith-Jones"
/// gives "smith" and "jones" as well as "smithjones"), a word broken across
/// a line by a hyphen read whole ("Wid-" and "gets" give "widgets"), and
/// letters spaced out as initials read together ("J. P." gives "jp").
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
    /// it read no words, or when it did not read every word of the
    /// document surely (see [`read_whole`]), so that what the document
    /// gives is not known.
    pub fn of(source: &DocumentSource) -> Option<Self> {
        if !read_whole(source) {
            return None;
        }
        let written = source
            .pages
            .iter()
            .flat_map(|page| page.text.split_whitespace())
            .collect::<Vec<_>>();
        let mut words = HashSet::new();
        for (index, run) in written.iter().enumerate() {
            words.extend(word_keys(run));
            // A word broken across a line - or a page - by a hyphen: "Wid-"
            // then "gets" is "Widgets" to the person reading it.
            if run.ends_with(&LINE_BREAK_HYPHENS[..])
                && let Some(next) = written.get(index + 1)
            {
                words.extend(word_keys(&format!("{run}{next}")));
            }
        }
        let keys = written
            .iter()
            .map(|run| HouseRule::key(&plain(run)))
            .filter(|key| !key.is_empty())
            .collect::<Vec<_>>();
        words.extend(initials(&keys));
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

/// Whether Intern read every word of `source`, and read it surely. Not
/// when any of it went unread or unsure:
///
/// - a page it could not read as text at all ([`barely_readable`]);
/// - a page sent to the model as an image that no OCR reading stands for:
///   the image is the page, and its text - a few characters, or a text
///   layer that reads as bad OCR - is not what a person sees on it. A
///   blank scanned page, which OCR read as having nothing on it, is read;
/// - any warning the reader raised about its text but a corrupt text layer
///   a scan's OCR reading took the place of (`NATIVE_TEXT_CORRUPT`): a page
///   or part of one it kept without reading it surely (`TEXT_UNREAD`: a
///   text layer that reads as bad OCR kept because OCR could not do
///   better, a page or an image on it that OCR was to read and could not,
///   or read without confidence), OCR unsure of a page (`LOW_OCR_CONFIDENCE`),
///   text cut at the reader's limits (`TEXT_TRUNCATED`) or left out by
///   design (`CONTENT_ELIDED`), and any warning this build does not know;
/// - a character the reader could not decode: a replacement character, or
///   a word written in a font's private characters, whose letters are not
///   known.
///
/// Every one of them can hide a word a person reading the document sees,
/// and a word the text does not give that the document does is the one
/// mistake the words must not make (see [`gives`]).
fn read_whole(source: &DocumentSource) -> bool {
    let unread_image = source.page_image.as_ref().is_some_and(|image| {
        source
            .pages
            .iter()
            .find(|page| page.page_number == image.page_number)
            .is_none_or(|page| {
                page.ocr_confidence.is_none()
                    && !(page.origin == PageOrigin::Ocr && page.text.trim().is_empty())
            })
    });
    !barely_readable(source)
        && !unread_image
        && source
            .parser_warnings
            .iter()
            .all(|warning| warning.code == "NATIVE_TEXT_CORRUPT")
        && !source.pages.iter().any(|page| undecoded(&page.text))
}

/// Whether `text` holds characters the reader could not decode: a
/// replacement character anywhere; a C1 control character, which no text
/// holds and a text file in an encoding it was not read in does; or a run
/// between spaces holding a private-use character beside a letter, a digit
/// or another of them - a word set in a font whose characters map to no
/// letters. A private-use character on its own is a bullet or a box, and
/// hides no word.
fn undecoded(text: &str) -> bool {
    let private = |character: char| {
        matches!(
            character,
            '\u{e000}'..='\u{f8ff}' | '\u{f0000}'..='\u{ffffd}' | '\u{100000}'..='\u{10fffd}'
        )
    };
    text.contains(|character| matches!(character, '\u{fffd}' | '\u{80}'..='\u{9f}'))
        || text.split_whitespace().any(|run| {
            run.chars().any(private)
                && run
                    .chars()
                    .filter(|character| private(*character) || character.is_alphanumeric())
                    .count()
                    > 1
        })
}

/// The words a learned spelling writes that the document's own word does
/// not hold, when it adds any: each way of reading the words of `rule.to`,
/// as written between spaces, split at punctuation as well, or run
/// together without spaces (see [`joined`]), each [`spellings`] way, less
/// every form of the words of `rule.from` (see [`forms`]), the legal forms
/// ([`is_legal_form`]) and the words that only join others
/// ([`JOINING_WORDS`]). Letters spaced out as initials are read together
/// as well ("J. P." as "jp"). A document gives what the rule adds when it
/// gives any of their words (see [`gives`]).
///
/// `None` - nothing a document could give in its own words - where any
/// reading leaves no word: a short form ("Acme" for "Acme Corporation"), a
/// change of case, accents, spacing or punctuation, a legal form added or
/// changed ("Acme Corporation" for "Acme", "ACME Corp" for "ACME
/// CORPORATION"); and for an acronym of the document's word ("DEQ" for
/// "Department of Environmental Quality"), which adds letters, not a party,
/// and whose letters a document gives beside the name as often as not. Not
/// for the words an acronym stands for ("American Broadcasting Company" for
/// "ABC"): they are words a document gives or does not, and one that spells
/// them out itself says nothing about the next "ABC", which may be ABC
/// Supply.
pub fn added(rule: &HouseRule) -> Option<Vec<Vec<String>>> {
    if is_acronym(&rule.from, &rule.to) {
        return None;
    }
    let own = forms(&rule.from).into_iter().collect::<HashSet<_>>();
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
    let mut readings = Vec::new();
    for to in spellings(&rule.to) {
        let words = words_of(&to);
        let pieces = pieces_of(&to);
        // Run together whole, the spelling adds nothing where that is a word
        // of the document's own ("Acme Widgets" for "AcmeWidgets").
        let joined = joined(&words);
        if joined.whole.iter().any(|word| own.contains(word)) {
            readings.push(Vec::new());
        } else if !joined.whole.is_empty() {
            readings.push(adds(joined.runs));
        }
        readings.push(adds(
            initials(&words).into_iter().chain(words.clone()).collect(),
        ));
        readings.push(adds(
            initials(&pieces)
                .into_iter()
                .chain(pieces.clone())
                .collect(),
        ));
    }
    (!readings.iter().any(Vec::is_empty)).then_some(readings)
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
/// words of `named`, the names its reading gives (its parties, its type),
/// read every way a rule's words are (see [`forms`]). A part is enough. A
/// document that writes "J. Doe" or "Ms. Doe" can make "John Smith, Jane
/// Doe" read as its own words to the person approving it, as surely as one
/// that writes "Jane Doe" or "Doe, Jane"; and asking for the whole as a run
/// of words is more than the words kept can answer. The price is a spelling
/// that adds two words or more, one of which the document gives on its
/// own, waiting for another document's approval. Without the words of its
/// text - a reading stored before they were kept, or a document Intern did
/// not read whole and surely (see [`DocumentWords::of`]) - it is not known,
/// and taken to give them: in doubt, no check.
pub fn gives(added: &[Vec<String>], text: Option<&DocumentWords>, named: &[&str]) -> bool {
    let Some(text) = text else {
        return true;
    };
    let named = named
        .iter()
        .flat_map(|name| forms(name))
        .collect::<HashSet<_>>();
    let given = |word: &str| named.contains(word) || text.gives(word);
    added
        .iter()
        .flatten()
        .any(|word| in_parts(word, MAX_PARTS, &given))
}

/// The most parts a document can write a word of a spelling in, apart
/// ("Smith Jones" for "SmithJones", "Van der Berg" for "Vanderberg"), and
/// the fewest letters each part has.
const MAX_PARTS: usize = 3;
const MIN_PART: usize = 3;

/// Whether `given` gives `word`, whole or in up to `parts` parts of
/// [`MIN_PART`] letters or more, one after another: a document that writes
/// "Smith Jones" gives "smithjones". The parts are looked for anywhere in
/// the document, not side by side - the words kept cannot say where they
/// were - which takes a document giving them apart as giving the word: in
/// doubt, no check.
fn in_parts(word: &str, parts: usize, given: &dyn Fn(&str) -> bool) -> bool {
    given(word)
        || (parts > 1
            && word.char_indices().skip(MIN_PART).any(|(at, _)| {
                let (first, rest) = word.split_at(at);
                rest.chars().count() >= MIN_PART && given(first) && in_parts(rest, parts - 1, given)
            }))
}

/// Whether `short` is an acronym of `long`: a single word, but for any
/// legal form ("IBM Corp", see [`is_legal_form`]) and any word `long` has
/// too ("KKR Holdings" for "Kohlberg Kravis Roberts Holdings"), of two
/// letters or more, made of the first letters of `long`'s words - all of
/// them, or leaving out the words that only join others or the company
/// forms ("DEQ" for "Department of Environmental Quality", "IBM" for
/// "International Business Machines Corporation"), each reading of the
/// words (see [`added`]) - or of its last words, leaving out a prefix in
/// front of them ("EPA" for "U.S. Environmental Protection Agency", "DEQ"
/// for "Oregon Department of Environmental Quality"). The letters run to
/// the last word: "DOE" is not "Department of Environmental Quality", nor
/// its "Department of". A "Group" or "Holdings" the long name does not
/// have is a word of the short one: "KKR Holdings" for "Kohlberg Kravis
/// Roberts" names the parent, and adds a word.
fn is_acronym(long: &str, short: &str) -> bool {
    let (long, short) = (&plain(long), &plain(short));
    let long_words = words_of(long)
        .into_iter()
        .chain(pieces_of(long))
        .collect::<HashSet<_>>();
    let short = words_of(short)
        .into_iter()
        .filter(|word| !is_legal_form(word) && !long_words.contains(word))
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
///
/// Only letters and digits are decomposed. A sign is left as it is, for
/// [`HouseRule::key`] to take out as it takes out punctuation: decomposed,
/// "™" was "TM" glued to the word before it, so "Widgets™" did not give
/// "widgets". A letter that does not decompose but is read as one that
/// does is written as that one: "Łukasz" is "Lukasz", "Søren" "Soren",
/// "Đorđe" "Dorde", "ß" "ss", "æ" "ae", "œ" "oe", "þ" "th", a dotless "ı"
/// an "i" (see [`plain_letter`]); and a modifier letter written as an
/// apostrophe ("Oʼbrien") is an apostrophe.
fn plain(value: &str) -> String {
    let mut plain = String::with_capacity(value.len());
    for character in value.chars() {
        if !character.is_alphanumeric() {
            if !is_combining_mark(character) {
                plain.push(character);
            }
            continue;
        }
        for part in std::iter::once(character)
            .nfkd()
            .filter(|part| !is_combining_mark(*part))
        {
            match plain_letter(part) {
                Some(letters) => plain.push_str(letters),
                None => plain.push(part),
            }
        }
    }
    plain
}

/// How a person types a letter that has no decomposition to one they
/// would type instead: the Latin letters written with a stroke, a bar or a
/// hook, the ligatures and old letters still in use, the dotless "i", the
/// Greek final sigma (written "σ" when the word is in capitals), and the
/// modifier letters written as an apostrophe or an accent, which are an
/// apostrophe. `None` for every other letter. Ŋ, ĸ and the letters of
/// other alphabets stay as they are.
fn plain_letter(letter: char) -> Option<&'static str> {
    Some(match letter {
        'ł' | 'Ł' | 'ƚ' | 'Ƚ' => "l",
        'ø' | 'Ø' => "o",
        'đ' | 'Đ' | 'ð' | 'Ð' | 'Ɖ' | 'ɖ' | 'ɗ' | 'Ɗ' => "d",
        'ħ' | 'Ħ' => "h",
        'ı' | 'ɨ' | 'Ɨ' => "i",
        'ß' | 'ẞ' => "ss",
        'æ' | 'Æ' => "ae",
        'œ' | 'Œ' => "oe",
        'þ' | 'Þ' => "th",
        'ŧ' | 'Ŧ' | 'ƭ' | 'Ƭ' | 'ʈ' | 'Ʈ' => "t",
        'ƀ' | 'Ƀ' | 'ɓ' | 'Ɓ' => "b",
        'ƈ' | 'Ƈ' | 'ȼ' | 'Ȼ' => "c",
        'ƒ' | 'Ƒ' => "f",
        'ǥ' | 'Ǥ' | 'ɠ' | 'Ɠ' => "g",
        'ȷ' | 'ɉ' | 'Ɉ' => "j",
        'ƙ' | 'Ƙ' => "k",
        'ƥ' | 'Ƥ' => "p",
        'ɍ' | 'Ɍ' => "r",
        'ʉ' | 'Ʉ' => "u",
        'ʋ' | 'Ʋ' => "v",
        'ƴ' | 'Ƴ' | 'ɏ' | 'Ɏ' => "y",
        'ƶ' | 'Ƶ' | 'ȥ' | 'Ȥ' => "z",
        'ⱥ' | 'Ⱥ' => "a",
        'ɇ' | 'Ɇ' => "e",
        'ς' => "σ",
        '\u{2b9}'..='\u{2c1}'
        | '\u{2c6}'..='\u{2d1}'
        | '\u{2ec}'
        | '\u{2ee}'
        | '\u{a78b}'
        | '\u{a78c}' => "'",
        _ => return None,
    })
}

/// The ways `value` is read [`plain`]: as written, and - where it holds a
/// letter commonly written out in two when it cannot be typed ("ä" as
/// "ae", "ö" and "ø" as "oe", "ü" as "ue", "å" as "aa") - written out that
/// way as well. "Müller", "Muller" and "Mueller" are one name to the
/// person approving it, so a document writing any of them gives the name
/// however a spelling writes it.
fn spellings(value: &str) -> Vec<String> {
    let composed = value.nfc().collect::<String>();
    let written_out = composed
        .chars()
        .map(|character| match character {
            'ä' => "ae".to_owned(),
            'Ä' => "Ae".to_owned(),
            'ö' | 'ø' => "oe".to_owned(),
            'Ö' | 'Ø' => "Oe".to_owned(),
            'ü' => "ue".to_owned(),
            'Ü' => "Ue".to_owned(),
            'å' => "aa".to_owned(),
            'Å' => "Aa".to_owned(),
            other => other.to_string(),
        })
        .collect::<String>();
    if written_out == composed {
        vec![plain(value)]
    } else {
        vec![plain(value), plain(&written_out)]
    }
}

/// Every word `value` gives, read every way (see [`spellings`]): as
/// written between spaces, split at punctuation, its initials read
/// together, and run together without spaces.
fn forms(value: &str) -> Vec<String> {
    spellings(value)
        .iter()
        .flat_map(|value| {
            let words = words_of(value);
            let pieces = pieces_of(value);
            initials(&words)
                .into_iter()
                .chain(initials(&pieces))
                .chain(joined(&words).runs)
                .chain(words)
                .chain(pieces)
        })
        .collect()
}

/// The words of `value`, read [`plain`] already, as written between
/// spaces, as house style compares spellings: "Smith-Jones, Inc." is
/// "smithjones" and "inc".
fn words_of(value: &str) -> Vec<String> {
    HouseRule::key(value)
        .split(' ')
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The words of `value`, read [`plain`] already, split at punctuation as
/// well: "Smith-Jones, Inc." is "smith", "jones" and "inc".
fn pieces_of(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(HouseRule::key)
        .filter(|word| !word.is_empty())
        .collect()
}

/// The most words of a name read run together as one ("Smith Jones" as
/// "smithjones"), past those of the whole name.
const MAX_JOINED: usize = 4;

/// Words run together without spaces (see [`joined`]).
struct Joined {
    /// All of them but the legal forms, and all of them but the legal forms
    /// and the words that only join others, when there are two or more to
    /// run together: "Acme Widgets, Inc." is "acmewidgets", "Bank of
    /// America" "bankofamerica" and "bankamerica".
    whole: Vec<String>,
    /// Those, and every run of two to [`MAX_JOINED`] of the words they are
    /// made of, one after another: "Acme Smith Jones" gives "acmesmith",
    /// "smithjones" and "acmesmithjones".
    runs: Vec<String>,
}

/// `words` run together without spaces. A document can write as one word
/// what a spelling writes as two ("SmithJones" for "Smith Jones"), and a
/// spelling that writes as two what the document writes as one adds no
/// word.
fn joined(words: &[String]) -> Joined {
    let mut joined = Joined {
        whole: Vec::new(),
        runs: Vec::new(),
    };
    for joining in [false, true] {
        let kept = words
            .iter()
            .filter(|word| !is_legal_form(word))
            .filter(|word| !joining || !JOINING_WORDS.contains(&word.as_str()))
            .map(String::as_str)
            .collect::<Vec<_>>();
        if kept.len() < 2 {
            continue;
        }
        joined.whole.push(kept.concat());
        joined.runs.push(kept.concat());
        for start in 0..kept.len() {
            for end in start + 2..=kept.len().min(start + MAX_JOINED) {
                joined.runs.push(kept[start..end].concat());
            }
        }
    }
    joined.whole.dedup();
    joined.runs.sort();
    joined.runs.dedup();
    joined
}

/// Letters spaced out as initials, read together: every run of two to
/// [`MAX_INITIALS`] words of one letter or digit each, one after another in
/// `words` - "J. P. Morgan" gives "jp", "U S A" gives "us", "sa" and
/// "usa", "3 M" gives "3m".
fn initials(words: &[String]) -> Vec<String> {
    let mut initials = Vec::new();
    let mut run: Vec<&str> = Vec::new();
    for word in words.iter().map(String::as_str).chain(std::iter::once("")) {
        if word.chars().count() == 1 {
            run.push(word);
            continue;
        }
        for start in 0..run.len() {
            for end in start + 2..=run.len().min(start + MAX_INITIALS) {
                initials.push(run[start..end].concat());
            }
        }
        run.clear();
    }
    initials
}

/// The words a run of characters between spaces gives, read both ways and
/// every [`spellings`] way.
fn word_keys(written: &str) -> Vec<String> {
    spellings(written)
        .into_iter()
        .flat_map(|written| {
            std::iter::once(HouseRule::key(&written))
                .chain(pieces_of(&written))
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>()
        })
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
            ("Lukasz Smith", "\u{141}ukasz Smith"),
            ("Mueller", "M\u{fc}ller"),
            ("Acme Widgets", "AcmeWidgets"),
            ("AcmeWidgets", "Acme Widgets, Inc."),
            ("JP Morgan", "J. P. Morgan"),
            ("Acme Widgets", "Acme Widgets\u{2122}"),
            ("Kohlberg Kravis Roberts Holdings", "KKR Holdings"),
        ] {
            assert_eq!(added(&party(from, to)), None, "{from} -> {to}");
        }
        // The words each reading adds as written between spaces and split
        // at punctuation, after those it adds run together.
        let words = |readings: Option<Vec<Vec<String>>>| {
            readings.map(|readings| {
                let [.., words, pieces] = readings.as_slice() else {
                    panic!("{readings:?}");
                };
                words.iter().chain(pieces).cloned().collect::<Vec<_>>()
            })
        };
        let strings = |words: &[&str]| words.iter().map(|word| (*word).to_owned()).collect();
        assert_eq!(
            words(added(&party("John Smith", "John Smith, Jane Doe"))),
            Some(strings(&["jane", "doe", "jane", "doe"]))
        );
        assert_eq!(
            words(added(&party("Smith", "John Smith"))),
            Some(strings(&["john", "john"]))
        );
        assert_eq!(
            words(added(&party("Acme", "Acme Holdings Trust"))),
            Some(strings(&["holdings", "trust", "holdings", "trust"]))
        );
        assert_eq!(
            words(added(&party("Acme", "Acme Holdings"))),
            Some(strings(&["holdings", "holdings"])),
            "the parent of Acme is not Acme"
        );
        assert_eq!(
            words(added(&party("Acme", "Acme Group, Inc."))),
            Some(strings(&["group", "group"]))
        );
        assert_eq!(
            words(added(&party("ABC", "American Broadcasting Company"))),
            Some(strings(&[
                "american",
                "broadcasting",
                "american",
                "broadcasting"
            ])),
            "the words an acronym stands for"
        );
        // Run together, as a document may write them, less the words of
        // the document's own run together.
        let joined = added(&party("John Smith", "John Smith, Jane Doe")).unwrap();
        assert!(joined[0].contains(&"janedoe".to_owned()), "{joined:?}");
        assert!(
            joined[0].contains(&"johnsmithjanedoe".to_owned()),
            "{joined:?}"
        );
        assert!(!joined[0].contains(&"johnsmith".to_owned()), "{joined:?}");
        // A "Holdings" or a "Group" beside an acronym names the parent: it
        // is no contraction of the document's word. It was left out of the
        // short side as a company form, and "KKR Holdings" for "Kohlberg
        // Kravis Roberts" added nothing.
        assert_eq!(
            words(added(&party("Kohlberg Kravis Roberts", "KKR Holdings"))),
            Some(strings(&["kkr", "holdings", "kkr", "holdings"]))
        );
        assert_eq!(
            words(added(&party("Kohlberg Kravis Roberts", "KKR Group"))),
            Some(strings(&["kkr", "group", "kkr", "group"]))
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

    /// The words of `rule`'s spelling, and whether a text writing `written`
    /// gives what it adds.
    fn given(from: &str, to: &str, written: &str) -> bool {
        let adds = added(&party(from, to)).unwrap_or_else(|| panic!("{from} -> {to} adds a word"));
        gives(&adds, Some(&text(written)), &[])
    }

    /// A sign beside a word is no part of it, as punctuation is not: "™"
    /// decomposed to "TM", and "Widgets™" in the text gave "widgetstm", not
    /// "widgets", so an agreement naming "Acme Widgets™" checked "Acme"
    /// written as "Acme Widgets".
    #[test]
    fn a_sign_beside_a_word_is_no_part_of_it() {
        for sign in ['\u{2122}', '\u{2120}', '\u{ae}', '\u{a9}'] {
            assert!(
                given(
                    "Acme",
                    "Acme Widgets",
                    &format!("Agreement with Acme Widgets{sign}, a division of Acme.")
                ),
                "{sign}"
            );
            assert!(
                given(
                    "Acme",
                    &format!("Acme Widgets{sign}"),
                    "Agreement with Acme Widgets, a division of Acme."
                ),
                "{sign}"
            );
        }
        assert!(!given(
            "Acme",
            "Acme Widgets",
            "Agreement with Acme, a company."
        ));
    }

    /// A letter Unicode does not decompose is the letter a person types for
    /// it, in the spelling and in the text alike: "Łukasz" in the text gives
    /// "Lukasz" typed in the spelling, and "Søren" "Soren". Each gave no
    /// word, so approving the agreement that names him checked the
    /// spelling that added his first name.
    #[test]
    fn a_letter_with_no_decomposition_is_the_letter_a_person_types_for_it() {
        for (typed, written) in [
            ("Lukasz", "\u{141}ukasz"),
            ("lukasz", "\u{142}ukasz"),
            ("Soren", "S\u{f8}ren"),
            ("SOREN", "S\u{d8}REN"),
            ("Dorde", "\u{110}or\u{111}e"),
            ("Gudrun", "Gu\u{f0}run"),
            ("GUDRUN", "GU\u{d0}RUN"),
            ("Hamrun", "\u{126}amrun"),
            ("hamrun", "\u{127}amrun"),
            ("Kirklareli", "K\u{131}rklareli"),
            ("Strasse", "Stra\u{df}e"),
            ("STRASSE", "STRA\u{1e9e}E"),
            ("Aesir", "\u{c6}sir"),
            ("Caesar", "C\u{e6}sar"),
            ("Oeuvre", "\u{152}uvre"),
            ("oeuvre", "\u{153}uvre"),
            ("Thorsson", "\u{de}orsson"),
            ("thorsson", "\u{fe}orsson"),
            ("Kostas", "\u{39a}\u{3ce}\u{3c3}\u{3c4}\u{3b1}\u{3c2}"),
        ] {
            // Greek is compared with Greek: the final sigma is a sigma.
            let typed = if typed == "Kostas" {
                "\u{39a}\u{3a9}\u{3a3}\u{3a4}\u{391}\u{3a3}"
            } else {
                typed
            };
            assert!(
                given(
                    "Smith",
                    &format!("{typed} Smith"),
                    &format!("Agreement with {written} Smith and Acme.")
                ),
                "{typed} typed, {written} written"
            );
            assert!(
                given(
                    "Smith",
                    &format!("{written} Smith"),
                    &format!("Agreement with {typed} Smith and Acme.")
                ),
                "{written} typed, {typed} written"
            );
            assert_eq!(
                added(&party(
                    &format!("{typed} Smith"),
                    &format!("{written} Smith")
                )),
                None,
                "{typed} -> {written}"
            );
        }
        assert!(!given(
            "Smith",
            "\u{141}ukasz Smith",
            "Agreement with Mr. Smith and Acme."
        ));
    }

    /// A word is given however the text writes its punctuation, its spacing
    /// and its letters, whichever way the spelling writes them - each pair
    /// of ways of writing it, in the spelling and in the text: apostrophes
    /// and quotes (a modifier letter apostrophe is a letter to Unicode, and
    /// was kept in the word), hyphens and dashes, a soft hyphen or a
    /// zero-width character inside the word, a word written as two or two
    /// as one, "&" for "and", initials dotted, run together or spaced out,
    /// a number and its letter apart, wide and superscript forms, and a
    /// letter written out ("Mueller" for "Müller").
    #[test]
    fn a_word_is_given_however_the_text_writes_its_punctuation_spacing_and_letters() {
        let classes: [&[&str]; 10] = [
            &[
                "O'Brien",
                "O\u{2019}Brien",
                "O\u{2bc}Brien",
                "O\u{2bb}Brien",
                "O\u{b4}Brien",
                "O`Brien",
                "O\u{2032}Brien",
                "O\u{ff07}Brien",
                "OBrien",
            ],
            &[
                "Smith-Jones",
                "Smith\u{2013}Jones",
                "Smith\u{2010}Jones",
                "Smith\u{2011}Jones",
                "Smith\u{2212}Jones",
                "Smith Jones",
                "SmithJones",
                "Smith\u{ad}Jones",
                "Smith\u{200b}Jones",
                "Smith\u{200d}Jones",
                "\u{feff}Smith-Jones",
            ],
            &[
                "Smith & Jones",
                "Smith and Jones",
                "Smith&Jones",
                "Smith + Jones",
            ],
            &["J.P.", "JP", "J P", "J. P.", "J.\u{a0}P."],
            &["3M", "3 M", "\u{ff13}\u{ff2d}", "3-M"],
            &["2nd", "2\u{207f}\u{1d48}", "\u{ff12}\u{ff4e}\u{ff44}"],
            &[
                "Widgets",
                "\u{ff37}\u{ff49}\u{ff44}\u{ff47}\u{ff45}\u{ff54}\u{ff53}",
                "WIDGETS",
                "\u{1d416}idgets",
            ],
            &[
                "M\u{fc}ller",
                "Mu\u{308}ller",
                "Mueller",
                "M\u{dc}LLER",
                "MUELLER",
            ],
            &["S\u{f8}ren", "Soeren", "S\u{d8}REN"],
            &["\u{c5}se", "Aase", "A\u{30a}se"],
        ];
        for class in classes {
            for typed in class {
                for written in class {
                    assert!(
                        given(
                            "Acme",
                            &format!("Acme {typed}"),
                            &format!("Agreement with Acme and {written}.")
                        ),
                        "{typed:?} typed, {written:?} written"
                    );
                }
            }
        }
        // Not a word the text does not give in any of its forms.
        for (to, written) in [
            ("Acme O'Brien", "Agreement with Acme and Brian."),
            ("Acme SmithJones", "Agreement with Acme and Smithson."),
            ("Acme JP", "Agreement with Acme and JQ."),
            ("Acme M\u{fc}ller", "Agreement with Acme and Miller."),
        ] {
            assert!(!given("Acme", to, written), "{to}: {written}");
        }
    }

    /// A word broken across a line by a hyphen - or across a page - is the
    /// whole word to the person reading it: "Wid-" at the end of a line and
    /// "gets" at the start of the next gave "wid" and "gets", never
    /// "widgets".
    #[test]
    fn a_word_broken_across_a_line_is_given_whole() {
        // Broken where the parts are too short to be looked for apart (see
        // `in_parts`).
        for broken in [
            "Agreement with Acme Wi-\ndgets and John Smith.",
            "Agreement with Acme Widget-\ns and John Smith.",
            "Agreement with Acme Wi\u{ad}\ndgets and John Smith.",
            "Agreement with Acme Wi\u{ac}\ndgets and John Smith.",
            "Agreement with Acme Wi\u{2010}\r\ndgets and John Smith.",
            "Agreement with Acme Wi- dgets and John Smith.",
        ] {
            assert!(given("Acme", "Acme Widgets", broken), "{broken:?}");
        }
        assert!(!given(
            "Acme",
            "Acme Widgets",
            "Agreement with Acme Wi and dgets."
        ));
        let pages = DocumentSource::from_pages(vec![
            SourcePage::new(1, "Agreement with Acme Wi-", PageOrigin::Native),
            SourcePage::new(2, "dgets and John Smith.", PageOrigin::Native),
        ]);
        assert!(gives(
            &added(&party("Acme", "Acme Widgets")).unwrap(),
            DocumentWords::of(&pages).as_ref(),
            &[]
        ));
    }

    /// A document Intern did not read whole and surely gives no words to go
    /// by: an image sent for a page no OCR reading stands for, a page or
    /// part of one kept without reading it surely, OCR unsure, text cut or
    /// left out, a character it could not decode. A blank scanned page
    /// carrying the image, a scan OCR read with confidence, and a corrupt
    /// text layer OCR read in its place give their words. The cover page
    /// of an agreement, a text layer of garbage sent as an image, left the
    /// words of the pages after it taken as everything the document gave.
    #[test]
    fn a_document_intern_did_not_read_whole_and_surely_gives_no_words() {
        let body = "This agreement is made by John Smith and Acme Corporation. The employee \
                    shall serve as warehouse supervisor and report to the operations manager. \
                    Compensation includes a base salary and standard benefits. Either party \
                    may end this agreement with thirty days written notice.";
        let image = |page_number| PageImage {
            page_number,
            media_type: "image/png".into(),
            bytes: vec![1, 2, 3],
        };
        let scanned = |text: &str, confidence| {
            let mut page = SourcePage::new(2, text, PageOrigin::Ocr);
            page.ocr_confidence = confidence;
            page
        };

        let mut cover = DocumentSource::from_pages(vec![
            SourcePage::new(1, "X7QZ0K4LMNB2RTY8WVU3JHG5FDS1", PageOrigin::Native),
            SourcePage::new(2, body, PageOrigin::Native),
        ]);
        cover.page_image = Some(image(1));
        assert_eq!(DocumentWords::of(&cover), None, "an unread cover page");
        let mut elsewhere = cover.clone();
        elsewhere.page_image = Some(image(9));
        assert_eq!(DocumentWords::of(&elsewhere), None, "an image of no page");

        let mut blank = DocumentSource::from_pages(vec![
            SourcePage::new(1, body, PageOrigin::Ocr),
            scanned("", None),
        ]);
        blank.pages[0].ocr_confidence = Some(91);
        blank.page_image = Some(image(2));
        assert!(DocumentWords::of(&blank).is_some(), "a blank page is read");
        let mut read = blank.clone();
        read.pages[1] = scanned(body, Some(88));
        assert!(DocumentWords::of(&read).is_some(), "a scan read surely");

        for code in [
            "TEXT_UNREAD",
            "LOW_OCR_CONFIDENCE",
            "TEXT_TRUNCATED",
            "CONTENT_ELIDED",
            "A_WARNING_FROM_A_NEWER_WORKER",
        ] {
            let mut warned =
                DocumentSource::from_pages(vec![SourcePage::new(1, body, PageOrigin::Native)]);
            warned.parser_warnings.push(ParserWarning::new(code, false));
            assert_eq!(DocumentWords::of(&warned), None, "{code}");
        }
        let mut replaced = read.clone();
        replaced
            .parser_warnings
            .push(ParserWarning::new("NATIVE_TEXT_CORRUPT", true));
        assert!(
            DocumentWords::of(&replaced).is_some(),
            "a corrupt layer OCR read in its place"
        );

        for undecoded in [
            "made by John Smith, Jane D\u{fffd}e and Acme Corporation.",
            "made by John Smith, \u{fffd}\u{fffd}\u{fffd} and Acme Corporation.",
            "made by John Smith, \u{e04a}\u{e061}\u{e06e}\u{e065} Doe and Acme.",
            "made by John Smith, Ja\u{f8ff}ne Doe and Acme Corporation.",
            "made by John Smith, Jane D\u{9d}e and Acme Corporation.",
        ] {
            assert_eq!(
                DocumentWords::of(&DocumentSource::from_pages(vec![SourcePage::new(
                    1,
                    undecoded,
                    PageOrigin::Native
                )])),
                None,
                "{undecoded:?}"
            );
        }
        assert!(
            DocumentWords::of(&DocumentSource::from_pages(vec![SourcePage::new(
                1,
                "\u{f0b7} made by John Smith and Acme Corporation.",
                PageOrigin::Native
            )]))
            .is_some(),
            "a bullet"
        );
    }
}
