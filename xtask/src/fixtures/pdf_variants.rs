//! Integrity variants of the clean one-column PDF: hidden, stuffed and mis-encoded text.

use super::content as c;
use super::pdf_layouts::{BLACK, BODY, WHITE, one_column};
use super::pdf_writer::{Font, Op, PAGE_W, PdfSpec};

/// The clean one-column resume plus one extra text run.
fn with_extra(size: f32, font: Font, rgb: [f32; 3], x: f32, y: f32, text: &str) -> PdfSpec {
    let mut s = one_column();
    s.pages[0].push(Op::Text {
        x,
        y,
        size,
        font,
        rgb,
        text: text.to_owned(),
    });
    s
}

pub fn hidden_white() -> PdfSpec {
    with_extra(BODY, Font::Regular, WHITE, 60.0, 60.0, c::STUFFED_KEYWORDS)
}

pub fn hidden_tiny() -> PdfSpec {
    with_extra(1.0, Font::Regular, BLACK, 60.0, 60.0, c::STUFFED_KEYWORDS)
}

pub fn off_page() -> PdfSpec {
    with_extra(
        BODY,
        Font::Regular,
        BLACK,
        PAGE_W + 100.0,
        400.0,
        c::STUFFED_KEYWORDS,
    )
}

pub fn metadata_only() -> PdfSpec {
    PdfSpec {
        keywords: Some(c::STUFFED_KEYWORDS.to_owned()),
        ..one_column()
    }
}

/// Rewrites every text run equal to `from` into `to`, drawn in `font`.
fn rewrite(mut s: PdfSpec, from: &str, to: &str, font: Font) -> PdfSpec {
    for op in s.pages.iter_mut().flatten() {
        if let Op::Text { text, font: f, .. } = op
            && text == from
        {
            *text = to.to_owned();
            *f = font;
        }
    }
    s
}

pub fn zero_width() -> PdfSpec {
    rewrite(
        one_column(),
        c::SKILLS[0],
        "Ru\u{200B}st, Go, Postgre\u{200B}SQL",
        Font::Mapped,
    )
}

pub fn ligature() -> PdfSpec {
    let lig = c::SUMMARY[1].replace("fi", "\u{FB01}");
    rewrite(one_column(), c::SUMMARY[1], &lig, Font::Mapped)
}

pub fn missing_to_unicode() -> PdfSpec {
    rewrite(one_column(), c::SKILLS[1], c::SKILLS[1], Font::Unmapped)
}
