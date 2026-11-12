//! The last few extractions, kept in memory.
//!
//! Every time the queue claims a document it extracts it again, and for a
//! scan that is OCR: up to a minute and a half for one long document. A
//! document is claimed more than once without its bytes changing - put back
//! because no model was loaded to ask, a second attempt after a model
//! failure, Re-analyze, a provisional reading read again by the accurate
//! model - and each of those read it again in full for the same text. The
//! memo hands back what the reader made of it the last time instead, within
//! one run of Intern: it starts empty, so the first reading of a document
//! after a restart - an update's included - is a fresh one.
//!
//! It is keyed by the queue item, the SHA-256 of the bytes the item was
//! checked against just before extraction, and the document's own path, so a
//! file that changed, or a document now at another path, never finds an
//! entry. The path is the document's, not the one the reader was handed: a
//! document admitted on a provider's proof is read from a private snapshot
//! made afresh for every claim, whose bytes are proven to be the ones the
//! hash names, and the reader makes the same of them wherever the copy lies.
//! It holds at most [`EXTRACTION_MEMO_ENTRIES`] documents and
//! [`EXTRACTION_MEMO_BYTES`] of them, the least recently used going first;
//! one larger than that is never kept. Nothing in it is written to disk, and
//! the queue drops a document's entry when the document is canceled or
//! removed - one at a time, or cleared away with Discard waiting or Clear
//! history - and every entry at shutdown, including what a reading still
//! under way would have kept ([`Reading`]).

use std::{
    collections::VecDeque,
    io,
    path::{Path, PathBuf},
    sync::Mutex,
};

use intern_engine::DocumentSource;

/// How many documents the memo holds at most.
pub const EXTRACTION_MEMO_ENTRIES: usize = 8;

/// How much extracted text, layout and page image the memo holds at most, in
/// bytes of the extraction as JSON: a generous measure of what it occupies
/// in memory. A long scanned contract with its layout is a few megabytes.
pub const EXTRACTION_MEMO_BYTES: usize = 32 * 1024 * 1024;

struct Entry {
    item_id: i64,
    source_hash: String,
    path: PathBuf,
    bytes: usize,
    source: DocumentSource,
}

impl Entry {
    fn is(&self, item_id: i64, source_hash: &str, path: &Path) -> bool {
        self.item_id == item_id && self.source_hash == source_hash && self.path == path
    }
}

#[derive(Default)]
struct Kept {
    /// Least recently used first.
    entries: VecDeque<Entry>,
    /// The readings under way, each as its item and the ticket it began
    /// with: what one makes of its document is kept only while it is still
    /// listed here.
    readings: Vec<(i64, u64)>,
    next_ticket: u64,
}

/// Extractions the queue may hand back instead of reading a document again.
pub struct ExtractionMemo {
    kept: Mutex<Kept>,
    max_entries: usize,
    max_bytes: usize,
}

impl Default for ExtractionMemo {
    fn default() -> Self {
        Self::with_limits(EXTRACTION_MEMO_ENTRIES, EXTRACTION_MEMO_BYTES)
    }
}

impl ExtractionMemo {
    /// A memo of at most `max_entries` documents and `max_bytes` of them.
    pub fn with_limits(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            kept: Mutex::new(Kept::default()),
            max_entries,
            max_bytes,
        }
    }

    /// What the reader made of item `item_id` the last time, when it read
    /// exactly these bytes of the document at `path`.
    pub fn get(&self, item_id: i64, source_hash: &str, path: &Path) -> Option<DocumentSource> {
        let mut kept = self.kept();
        let found = kept
            .entries
            .iter()
            .position(|entry| entry.is(item_id, source_hash, path))?;
        let entry = kept.entries.remove(found)?;
        let source = entry.source.clone();
        kept.entries.push_back(entry);
        Some(source)
    }

    /// Begins a reading of item `item_id`, before the reader is asked: what
    /// it makes of the document is kept ([`Reading::keep`]) only if nothing
    /// forgot the item while it read - a cancel or a removal landing during
    /// a minute of OCR drops the entry the reading has not made yet, too.
    /// Another reading of the same item begun meanwhile takes its place.
    pub fn reading(&self, item_id: i64) -> Reading<'_> {
        let mut kept = self.kept();
        let ticket = kept.next_ticket;
        kept.next_ticket = kept.next_ticket.wrapping_add(1);
        kept.readings.retain(|(id, _)| *id != item_id);
        kept.readings.push((item_id, ticket));
        Reading {
            memo: self,
            item_id,
            ticket,
        }
    }

    /// Drops whatever is kept for item `item_id`, and whatever a reading of
    /// it under way would keep.
    pub fn forget(&self, item_id: i64) {
        let mut kept = self.kept();
        kept.entries.retain(|entry| entry.item_id != item_id);
        kept.readings.retain(|(id, _)| *id != item_id);
    }

    /// The items anything is kept for, or a reading of which is under way,
    /// each once.
    pub fn items(&self) -> Vec<i64> {
        let kept = self.kept();
        let mut items = kept
            .entries
            .iter()
            .map(|entry| entry.item_id)
            .chain(kept.readings.iter().map(|(item_id, _)| *item_id))
            .collect::<Vec<_>>();
        items.sort_unstable();
        items.dedup();
        items
    }

    /// Drops everything, and whatever the readings under way would keep.
    pub fn clear(&self) {
        let mut kept = self.kept();
        kept.entries.clear();
        kept.readings.clear();
    }

    /// How many documents are kept, and how many bytes of them.
    pub fn held(&self) -> (usize, usize) {
        let kept = self.kept();
        (
            kept.entries.len(),
            kept.entries.iter().map(|entry| entry.bytes).sum(),
        )
    }

    /// Keeps what `reading` made of its document, in place of anything kept
    /// for the item before, and lets the least recently used documents go
    /// until the limits hold - unless the item was forgotten since the
    /// reading began. A document larger than the whole memo is not kept, and
    /// nothing is let go for it.
    fn keep(&self, reading: &Reading<'_>, source_hash: &str, path: &Path, source: &DocumentSource) {
        let bytes = json_bytes(source);
        let mut kept = self.kept();
        if !kept.readings.contains(&(reading.item_id, reading.ticket)) {
            return;
        }
        kept.entries
            .retain(|entry| entry.item_id != reading.item_id);
        if bytes > self.max_bytes || self.max_entries == 0 {
            return;
        }
        let mut held = kept.entries.iter().map(|entry| entry.bytes).sum::<usize>();
        while kept.entries.len() >= self.max_entries || held + bytes > self.max_bytes {
            match kept.entries.pop_front() {
                Some(evicted) => held -= evicted.bytes,
                None => break,
            }
        }
        kept.entries.push_back(Entry {
            item_id: reading.item_id,
            source_hash: source_hash.to_owned(),
            path: path.to_path_buf(),
            bytes,
            source: source.clone(),
        });
    }

    fn kept(&self) -> std::sync::MutexGuard<'_, Kept> {
        // A panic while the lock was held left at worst an entry too few or
        // too many; every entry is still whole.
        self.kept
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// One reading of a document under way ([`ExtractionMemo::reading`]). It
/// ends when it is dropped, whatever came of it.
pub struct Reading<'a> {
    memo: &'a ExtractionMemo,
    item_id: i64,
    ticket: u64,
}

impl Reading<'_> {
    /// Keeps what the reader made of the document, the bytes `source_hash`
    /// of the document at `path`, unless its item was forgotten since the
    /// reading began.
    pub fn keep(&self, source_hash: &str, path: &Path, source: &DocumentSource) {
        self.memo.keep(self, source_hash, path, source);
    }
}

impl Drop for Reading<'_> {
    fn drop(&mut self) {
        let mut kept = self.memo.kept();
        let this = (self.item_id, self.ticket);
        kept.readings.retain(|reading| *reading != this);
    }
}

/// The size of `source` as JSON, counted without writing it anywhere.
fn json_bytes(source: &DocumentSource) -> usize {
    let mut counter = ByteCounter(0);
    // Counting cannot fail; a source that could not be written is counted
    // as larger than any memo, and so never kept.
    match serde_json::to_writer(&mut counter, source) {
        Ok(()) => counter.0,
        Err(_) => usize::MAX,
    }
}

struct ByteCounter(usize);

impl io::Write for ByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(buffer.len());
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use intern_engine::{PageImage, PageOrigin, SourcePage};

    use super::*;

    fn text(words: &str) -> DocumentSource {
        DocumentSource::from_pages(vec![SourcePage::new(1, words, PageOrigin::Ocr)])
    }

    /// A document comes back only for the item, the bytes and the path it
    /// was read for: a changed file, the same text under another item, or
    /// the same item now at another path finds nothing.
    #[test]
    fn an_extraction_comes_back_only_for_the_same_bytes_at_the_same_path() {
        let memo = ExtractionMemo::default();
        let path = Path::new("/inbox/scan.pdf");
        let source = text("INVOICE 1001");
        memo.reading(7).keep("aaaa", path, &source);

        assert_eq!(memo.get(7, "aaaa", path), Some(source.clone()));
        assert_eq!(memo.get(7, "bbbb", path), None, "the file changed");
        assert_eq!(memo.get(8, "aaaa", path), None, "another item");
        assert_eq!(
            memo.get(7, "aaaa", Path::new("/inbox/NEEDS REVIEW - scan.pdf")),
            None
        );

        // A new reading of the item replaces the old one, whatever it was.
        memo.reading(7).keep("bbbb", path, &text("INVOICE 1002"));
        assert_eq!(memo.get(7, "aaaa", path), None);
        assert_eq!(memo.get(7, "bbbb", path), Some(text("INVOICE 1002")));
        assert_eq!(memo.held().0, 1);

        memo.forget(7);
        assert_eq!(memo.get(7, "bbbb", path), None);
        memo.reading(9).keep("cccc", path, &source);
        memo.clear();
        assert_eq!(memo.held(), (0, 0));
    }

    /// The items listed are those an entry is kept for and those a reading
    /// is under way for, each once, until they are forgotten.
    #[test]
    fn the_items_listed_are_those_kept_or_being_read() {
        let memo = ExtractionMemo::default();
        let path = Path::new("/inbox/scan.pdf");
        memo.reading(7).keep("aaaa", path, &text("INVOICE 1001"));
        let again = memo.reading(7);
        let first = memo.reading(3);
        assert_eq!(memo.items(), vec![3, 7]);
        memo.forget(7);
        assert_eq!(memo.items(), vec![3]);
        drop((again, first));
        assert!(memo.items().is_empty());
    }

    /// The limits hold: the least recently used document goes first, a
    /// document looked up is used again, and one larger than the whole memo
    /// is not kept and pushes nothing out.
    #[test]
    fn the_memo_keeps_to_its_limits_least_recently_used_first() {
        let one = text("ONE");
        let size = json_bytes(&one);
        let memo = ExtractionMemo::with_limits(3, size * 3);
        let path = Path::new("/inbox/a.pdf");
        for id in 1..=3 {
            memo.reading(id)
                .keep("h", path, &text(["ONE", "TWO", "SIX"][id as usize - 1]));
        }
        assert_eq!(memo.held(), (3, size * 3));
        // Item 1 is used, so item 2 is the least recently used.
        assert!(memo.get(1, "h", path).is_some());
        memo.reading(4).keep("h", path, &text("TEN"));
        assert_eq!(memo.held(), (3, size * 3));
        assert!(
            memo.get(2, "h", path).is_none(),
            "the least recently used went"
        );
        for id in [1, 3, 4] {
            assert!(memo.get(id, "h", path).is_some(), "item {id} stayed");
        }

        let mut scan = text("ONE");
        scan.page_image = Some(PageImage {
            page_number: 1,
            media_type: "image/png".into(),
            bytes: vec![0; size * 3],
        });
        memo.reading(5).keep("h", path, &scan);
        assert!(memo.get(5, "h", path).is_none(), "too large to keep");
        assert_eq!(memo.held().0, 3, "and nothing went for it");

        // Bytes bind as well as entries: room for eight documents, but only
        // for two long ones.
        let mut long = text("ONE");
        long.pages.push(SourcePage::new(2, "TWO", PageOrigin::Ocr));
        let long_size = json_bytes(&long);
        let memo = ExtractionMemo::with_limits(8, long_size * 2);
        memo.reading(1).keep("h", path, &one);
        memo.reading(2).keep("h", path, &long);
        assert_eq!(memo.held(), (2, size + long_size));
        memo.reading(3).keep("h", path, &long);
        assert_eq!(memo.held(), (2, long_size * 2));
        assert!(memo.get(1, "h", path).is_none(), "made room for");
        assert!(memo.get(2, "h", path).is_some() && memo.get(3, "h", path).is_some());
    }

    /// A reading keeps nothing when its item was forgotten while it read -
    /// a cancel or a removal during a long OCR - or the memo cleared, or
    /// another reading of the item begun since; and a reading that ends,
    /// whatever came of it, leaves nothing behind.
    #[test]
    fn a_reading_keeps_nothing_for_an_item_forgotten_while_it_read() {
        let memo = ExtractionMemo::default();
        let path = Path::new("/inbox/scan.pdf");
        let source = text("INVOICE 1001");

        let canceled = memo.reading(7);
        memo.forget(7);
        canceled.keep("aaaa", path, &source);
        assert_eq!(memo.get(7, "aaaa", path), None, "canceled while read");

        let stopping = memo.reading(7);
        memo.clear();
        stopping.keep("aaaa", path, &source);
        assert_eq!(memo.get(7, "aaaa", path), None, "shut down while read");

        let earlier = memo.reading(7);
        let later = memo.reading(7);
        earlier.keep("aaaa", path, &source);
        assert_eq!(memo.get(7, "aaaa", path), None, "read again since");
        later.keep("aaaa", path, &source);
        assert_eq!(memo.get(7, "aaaa", path), Some(source.clone()));

        // Another item's cancel takes nothing from this one's reading.
        let reading = memo.reading(8);
        memo.forget(9);
        reading.keep("bbbb", path, &source);
        assert_eq!(memo.get(8, "bbbb", path), Some(source));

        drop((canceled, stopping, earlier, later, reading));
        assert!(memo.kept().readings.is_empty(), "every reading ended");
    }
}
