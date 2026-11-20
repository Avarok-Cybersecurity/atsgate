//! Structural and integrity signals, each with a stable rule code and evidence.

use crate::model::{BBox, SpanId};

/// Why a piece of text is hidden from a human reader while a parser still reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HiddenReason {
    White,
    Tiny,
    ZeroWidth,
    OffPage,
    /// DOCX `w:vanish`.
    Vanished,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignalKind {
    /// Side-by-side text columns: row-by-row parsers interleave them.
    MultiColumn {
        columns: u32,
    },
    /// Reading order and stored order disagree (text boxes, out-of-flow blocks).
    ReadingOrderDisagreement {
        displaced_spans: u32,
    },
    Table,
    TextBox,
    HeaderFooterContent,
    ImageOnlyPage,
    /// A composite or Type3 font that is used without a ToUnicode map.
    MissingToUnicode {
        font: String,
    },
    /// Ligature code points (U+FB00..U+FB06) that break keyword matching.
    LigatureGlyphs,
    /// U+FFFD replacement characters: glyphs that could not be mapped to Unicode.
    UnmappedGlyphs,
    HiddenText(HiddenReason),
    /// Terms present only in document metadata, never in the visible text.
    MetadataOnlyText {
        field: &'static str,
    },
}

impl SignalKind {
    /// A stable rule id, used in reports and tests.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MultiColumn { .. } => "layout.multi_column",
            Self::ReadingOrderDisagreement { .. } => "layout.reading_order_disagreement",
            Self::Table => "layout.table",
            Self::TextBox => "layout.text_box",
            Self::HeaderFooterContent => "layout.header_footer_content",
            Self::ImageOnlyPage => "layout.image_only_page",
            Self::MissingToUnicode { .. } => "glyphs.missing_tounicode",
            Self::LigatureGlyphs => "glyphs.ligature",
            Self::UnmappedGlyphs => "glyphs.unmapped",
            Self::HiddenText(HiddenReason::White) => "integrity.hidden.white",
            Self::HiddenText(HiddenReason::Tiny) => "integrity.hidden.tiny",
            Self::HiddenText(HiddenReason::ZeroWidth) => "integrity.hidden.zero_width",
            Self::HiddenText(HiddenReason::OffPage) => "integrity.hidden.off_page",
            Self::HiddenText(HiddenReason::Vanished) => "integrity.hidden.vanished",
            Self::MetadataOnlyText { .. } => "integrity.metadata_only",
        }
    }

    /// Integrity flags are reported and fail a gate; they never add points.
    pub fn is_integrity(&self) -> bool {
        matches!(self, Self::HiddenText(_) | Self::MetadataOnlyText { .. })
    }
}

/// What a signal points at: a span, a box, or text that never became a span.
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    pub text: String,
    pub span: Option<SpanId>,
    pub bbox: Option<BBox>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub kind: SignalKind,
    /// 1-based page, when the signal belongs to one page.
    pub page: Option<u32>,
    pub evidence: Vec<Evidence>,
}
