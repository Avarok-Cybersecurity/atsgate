//! Detection thresholds. Callers pass them explicitly; [`Thresholds::STANDARD`] documents its choices.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    /// Text smaller than this many points is "tiny".
    pub tiny_font_pt: f32,
    /// Text whose R, G and B are all at least this is "white" (assumes a white page).
    pub white_min_channel: f32,
    /// Fraction of the page height at the top that counts as the header band.
    pub header_band: f32,
    /// Fraction of the page height at the bottom that counts as the footer band.
    pub footer_band: f32,
    /// Minimum empty horizontal gap, in points, between two text columns.
    pub column_gap_pt: f32,
    /// Two spans share a baseline when their baselines differ by at most this many points.
    pub baseline_tolerance_pt: f32,
    /// A vertical gutter separates columns only when each side has at least this many rows
    /// that the other side lacks.
    pub min_column_rows: u32,
    /// Reading order "disagrees" with stored order when at least this many spans are displaced.
    pub min_displaced_spans: u32,
    /// The largest decompressed DOCX part accepted; a larger one is an error, not a truncation.
    pub max_docx_part_bytes: u64,
}

impl Thresholds {
    /// The thresholds ATSGate ships with.
    ///
    /// - 4 pt: below the smallest size used for legible print (footnotes run 6-8 pt), so smaller
    ///   text is there for a parser, not a reader.
    /// - 0.95 per channel: near-white that a reader cannot see on a white page.
    /// - 6% bands: about the top and bottom 47 pt of a US Letter page, inside the usual 0.5-1 in
    ///   margins where word processors place headers and footers.
    /// - 12 pt gutter: narrower than any multi-column resume gutter seen in practice, wider than a
    ///   word space at body sizes.
    /// - 0.5 pt baseline tolerance: text on one row shares a baseline exactly in generated PDFs.
    /// - 2 unaligned rows per side: a lone page number or a right-aligned date column is not a
    ///   text column.
    /// - 2 displaced spans: one displaced span is common noise (a late-drawn page number).
    /// - 32 MiB per DOCX part: a resume's document.xml is kilobytes; this bounds a zip bomb.
    pub const STANDARD: Thresholds = Thresholds {
        tiny_font_pt: 4.0,
        white_min_channel: 0.95,
        header_band: 0.06,
        footer_band: 0.06,
        column_gap_pt: 12.0,
        baseline_tolerance_pt: 0.5,
        min_column_rows: 2,
        min_displaced_spans: 2,
        max_docx_part_bytes: 32 * 1024 * 1024,
    };
}
