//! The values the stages hand one another. Owned by no single stage, so
//! every stage can be written and tested against them on its own.

/// An 8-bit grayscale image, row-major, one byte a pixel: 0 is black ink,
/// 255 white paper, as OpenCV's `IMREAD_GRAYSCALE` reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gray {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Gray {
    pub fn new(width: usize, height: usize, data: Vec<u8>) -> Self {
        assert_eq!(data.len(), width * height, "gray image size");
        Self {
            width,
            height,
            data,
        }
    }

    pub fn filled(width: usize, height: usize, value: u8) -> Self {
        Self::new(width, height, vec![value; width * height])
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.data[y * self.width + x]
    }
}

/// A detected line: its box in page pixels and the detector's mean core
/// probability (spec §5.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub score: f32,
}

/// What one emitted character of a line is: a recognizer class id (the base
/// recognizer) or a decoded character (the script group recognizers).
#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Id(u32),
    Char(String),
}

/// One emitted character of a line and the x position, in page pixels, of
/// the time step that emitted it (spec §6.5).
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub x: f64,
}

/// The recognizer's reading of one line (spec §6.5): text is empty when
/// `conf` is under 0.5, and the line keeps its box.
#[derive(Clone, Debug, PartialEq)]
pub struct RecLine {
    pub text: String,
    pub conf: f32,
    pub tokens: Vec<Token>,
}

/// The reading-order network's outputs for `n` lines (spec §1, §8.5):
/// `a[i * n + j] > 0` means line i comes before line j, `cls` holds ten
/// block-class logits a line, `same[i * n + j] > 0` that the two lines
/// share a block.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderOutputs {
    pub n: usize,
    pub a: Vec<f32>,
    pub cls: Vec<f32>,
    pub same: Vec<f32>,
}

/// The ten block classes, in the network's class order (spec §8.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockClass {
    Text,
    SectionHeader,
    PageHeader,
    PageFooter,
    ListGroup,
    Table,
    Caption,
    Footnote,
    TableOfContents,
    Form,
}

impl BlockClass {
    pub const ALL: [BlockClass; 10] = [
        BlockClass::Text,
        BlockClass::SectionHeader,
        BlockClass::PageHeader,
        BlockClass::PageFooter,
        BlockClass::ListGroup,
        BlockClass::Table,
        BlockClass::Caption,
        BlockClass::Footnote,
        BlockClass::TableOfContents,
        BlockClass::Form,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL[index]
    }

    /// The reference package's label, as its JSON output writes it.
    pub fn label(self) -> &'static str {
        match self {
            BlockClass::Text => "Text",
            BlockClass::SectionHeader => "Section-Header",
            BlockClass::PageHeader => "Page-Header",
            BlockClass::PageFooter => "Page-Footer",
            BlockClass::ListGroup => "List-Group",
            BlockClass::Table => "Table",
            BlockClass::Caption => "Caption",
            BlockClass::Footnote => "Footnote",
            BlockClass::TableOfContents => "Table-Of-Contents",
            BlockClass::Form => "Form",
        }
    }
}

/// One line of the finished page, in reading order. `bbox` is
/// `[x0, y0, x1, y1]` in pixels of the upright (turned, unlevelled) page,
/// rounded to 0.1 as the reference rounds it; `conf` is 0-1.
#[derive(Clone, Debug, PartialEq)]
pub struct PageLine {
    pub bbox: [f64; 4],
    pub text: String,
    pub conf: f64,
}

/// One block of the finished page, in reading order: its class, the box
/// around its lines, its text (lines joined by newlines), and the indices
/// into [`PageReading::lines`] of the lines it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct PageBlock {
    pub class: BlockClass,
    pub bbox: [f64; 4],
    pub text: String,
    pub lines: Vec<usize>,
}

/// A page as Indra reads it (the reference `Page`).
#[derive(Clone, Debug, PartialEq)]
pub struct PageReading {
    /// Degrees counter-clockwise the content was turned, a multiple of 90.
    pub rotation: u32,
    /// The small tilt straightened, degrees, rounded to 0.01.
    pub skew: f64,
    pub lines: Vec<PageLine>,
    pub blocks: Vec<PageBlock>,
    /// Headings, paragraphs, lists and tables; page headers and footers left
    /// out.
    pub markdown: String,
    /// One line a detected line, in reading order.
    pub text: String,
}
