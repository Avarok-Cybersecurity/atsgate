//! The extracted document: pages, spans, lines in reading order, and metadata.

/// The input format. Callers state it explicitly; nothing is guessed from file names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Pdf,
    Docx,
    Html,
    Markdown,
    Text,
}

/// An axis-aligned box in PDF user space: origin bottom-left, y grows upwards, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BBox {
    pub fn width(&self) -> f32 {
        self.x1 - self.x0
    }
    pub fn height(&self) -> f32 {
        self.y1 - self.y0
    }
    pub fn center(&self) -> (f32, f32) {
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }
    pub fn contains_point(&self, (x, y): (f32, f32)) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }
    pub fn union(&self, o: &BBox) -> BBox {
        BBox {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }
}

/// An sRGB colour with channels in 0..=1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// Where on the page, or in the package, a span lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Body,
    Table,
    TextBox,
    Header,
    Footer,
}

/// Index of a span in [`Document::spans`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpanId(pub u32);

/// A run of text with uniform style, in the order the source stored it.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub id: SpanId,
    pub text: String,
    /// 1-based page number; flow formats (DOCX, HTML, Markdown, text) report page 1.
    pub page: u32,
    /// Present for PDF; flow formats have no geometry.
    pub bbox: Option<BBox>,
    pub font_size: Option<f32>,
    pub color: Option<Rgb>,
    pub region: Region,
}

/// One line of text in reading order, built from one or more spans.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub page: u32,
    pub bbox: Option<BBox>,
    pub spans: Vec<SpanId>,
    pub region: Region,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Page {
    pub number: u32,
    /// Width and height in points; `None` for flow formats.
    pub size: Option<(f32, f32)>,
    pub image_count: u32,
    pub span_count: u32,
}

/// Document-level metadata fields that a parser may read but a reader never sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataField {
    pub name: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub format: Format,
    pub pages: Vec<Page>,
    /// Spans in source (content-stream or document) order.
    pub spans: Vec<Span>,
    /// Lines in reading order.
    pub lines: Vec<Line>,
    pub metadata: Vec<MetadataField>,
    pub signals: Vec<crate::Signal>,
}

impl Document {
    /// Reading-order text: one line per [`Line`].
    pub fn reading_text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Stream-order text: one span per line, in the order the source stored them.
    pub fn stream_text(&self) -> String {
        self.spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn span(&self, id: SpanId) -> &Span {
        &self.spans[id.0 as usize]
    }
}
