//! The last few extractions, kept in memory.
//!
//! Every time the queue claims a document it extracts it again, and for a
//! scan that is OCR: up to a minute and a half for one long document. A
//! document is claimed more than once without its bytes changing - put back
//! because no model was loaded to ask, a second attempt after a model
//! failure, Re-analyze, a provisional reading read again by the accurate
//! model, a waiting document validated again after an update - and each of
//! those read it again in full for the same text. The memo hands back what
//! the reader made of it the last time instead.
//!
//! It is keyed by the queue item, the SHA-256 of the bytes the item was
//! checked against just before extraction, and the path that would be read,
//! so a file that changed, or a document read from anywhere else, never
//! finds an entry. It holds at most [`EXTRACTION_MEMO_ENTRIES`] documents and
//! [`EXTRACTION_MEMO_BYTES`] of them, the least recently used going first;
//! one larger than that is never kept. Nothing in it is written to disk, and
//! the queue drops a document's entry when the document is canceled or
//! removed, and every entry at shutdown.

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

/// Extractions the queue may hand back instead of reading a document again.
pub struct ExtractionMemo {
    /// Least recently used first.
    entries: Mutex<VecDeque<Entry>>,
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
            entries: Mutex::new(VecDeque::new()),
            max_entries,
            max_bytes,
        }
    }

    /// What the reader made of item `item_id` the last time, when it read
    /// exactly these bytes from exactly this path.
    pub fn get(&self, item_id: i64, source_hash: &str, path: &Path) -> Option<DocumentSource> {
        let mut entries = self.entries();
        let found = entries
            .iter()
            .position(|entry| entry.is(item_id, source_hash, path))?;
        let entry = entries.remove(found)?;
        let source = entry.source.clone();
        entries.push_back(entry);
        Some(source)
    }

    /// Keeps what the reader made of item `item_id`, in place of anything
    /// kept for it before, and lets the least recently used documents go
    /// until the limits hold. A document larger than the whole memo is not
    /// kept, and nothing is let go for it.
    pub fn keep(&self, item_id: i64, source_hash: &str, path: &Path, source: &DocumentSource) {
        let bytes = json_bytes(source);
        let mut entries = self.entries();
        entries.retain(|entry| entry.item_id != item_id);
        if bytes > self.max_bytes || self.max_entries == 0 {
            return;
        }
        let mut held = entries.iter().map(|entry| entry.bytes).sum::<usize>();
        while entries.len() >= self.max_entries || held + bytes > self.max_bytes {
            match entries.pop_front() {
                Some(evicted) => held -= evicted.bytes,
                None => break,
            }
        }
        entries.push_back(Entry {
            item_id,
            source_hash: source_hash.to_owned(),
            path: path.to_path_buf(),
            bytes,
            source: source.clone(),
        });
    }

    /// Drops whatever is kept for item `item_id`.
    pub fn forget(&self, item_id: i64) {
        self.entries().retain(|entry| entry.item_id != item_id);
    }

    /// Drops everything.
    pub fn clear(&self) {
        self.entries().clear();
    }

    /// How many documents are kept, and how many bytes of them.
    pub fn held(&self) -> (usize, usize) {
        let entries = self.entries();
        (entries.len(), entries.iter().map(|entry| entry.bytes).sum())
    }

    fn entries(&self) -> std::sync::MutexGuard<'_, VecDeque<Entry>> {
        // A panic while the lock was held left at worst an entry too few or
        // too many; every entry is still whole.
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
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
    /// the same item read from elsewhere finds nothing.
    #[test]
    fn an_extraction_comes_back_only_for_the_same_bytes_at_the_same_path() {
        let memo = ExtractionMemo::default();
        let path = Path::new("/inbox/scan.pdf");
        let source = text("INVOICE 1001");
        memo.keep(7, "aaaa", path, &source);

        assert_eq!(memo.get(7, "aaaa", path), Some(source.clone()));
        assert_eq!(memo.get(7, "bbbb", path), None, "the file changed");
        assert_eq!(memo.get(8, "aaaa", path), None, "another item");
        assert_eq!(
            memo.get(7, "aaaa", Path::new("/inbox/NEEDS REVIEW - scan.pdf")),
            None
        );

        // A new reading of the item replaces the old one, whatever it was.
        memo.keep(7, "bbbb", path, &text("INVOICE 1002"));
        assert_eq!(memo.get(7, "aaaa", path), None);
        assert_eq!(memo.get(7, "bbbb", path), Some(text("INVOICE 1002")));
        assert_eq!(memo.held().0, 1);

        memo.forget(7);
        assert_eq!(memo.get(7, "bbbb", path), None);
        memo.keep(9, "cccc", path, &source);
        memo.clear();
        assert_eq!(memo.held(), (0, 0));
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
            memo.keep(id, "h", path, &text(["ONE", "TWO", "SIX"][id as usize - 1]));
        }
        assert_eq!(memo.held(), (3, size * 3));
        // Item 1 is used, so item 2 is the least recently used.
        assert!(memo.get(1, "h", path).is_some());
        memo.keep(4, "h", path, &text("TEN"));
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
        memo.keep(5, "h", path, &scan);
        assert!(memo.get(5, "h", path).is_none(), "too large to keep");
        assert_eq!(memo.held().0, 3, "and nothing went for it");

        // Bytes bind as well as entries: room for eight documents, but only
        // for two long ones.
        let mut long = text("ONE");
        long.pages.push(SourcePage::new(2, "TWO", PageOrigin::Ocr));
        let long_size = json_bytes(&long);
        let memo = ExtractionMemo::with_limits(8, long_size * 2);
        memo.keep(1, "h", path, &one);
        memo.keep(2, "h", path, &long);
        assert_eq!(memo.held(), (2, size + long_size));
        memo.keep(3, "h", path, &long);
        assert_eq!(memo.held(), (2, long_size * 2));
        assert!(memo.get(1, "h", path).is_none(), "made room for");
        assert!(memo.get(2, "h", path).is_some() && memo.get(3, "h", path).is_some());
    }
}
