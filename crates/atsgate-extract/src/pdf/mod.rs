//! PDF: pdf_oxide supplies positioned glyphs (decoded through ToUnicode and font metrics);
//! lopdf supplies the stored drawing order, ruling paths, images, fonts and metadata.
//! Reading order is ATSGate's own (see `layout`): pdf_oxide's column-aware order split
//! right-aligned dates into a column of their own and interleaved two-column pages.

mod content;
mod glyphs;
mod runs;

use crate::error::ExtractError;
use crate::integrity::{self, SignalSet};
use crate::layout::{self, frames, order};
use crate::model::{BBox, Document, Format, MetadataField, Page, Region, Span, SpanId};
use crate::signals::{Evidence, HiddenReason, SignalKind};
use crate::thresholds::Thresholds;
use pdf_oxide::document::PdfDocument;

fn err(e: impl std::fmt::Display) -> ExtractError {
    ExtractError::Pdf(e.to_string())
}

fn off_page(b: &BBox, (x0, y0, x1, y1): (f32, f32, f32, f32)) -> bool {
    b.x1 <= x0 || b.x0 >= x1 || b.y1 <= y0 || b.y0 >= y1
}

fn metadata(doc: &lopdf::Document) -> Vec<MetadataField> {
    let info = doc
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|o| doc.dereference(o).ok())
        .and_then(|(_, o)| o.as_dict().ok());
    let Some(info) = info else { return Vec::new() };
    [
        ("Title", "pdf.title"),
        ("Subject", "pdf.subject"),
        ("Keywords", "pdf.keywords"),
    ]
    .into_iter()
    .filter_map(|(key, name)| {
        let value = lopdf::decode_text_string(info.get(key.as_bytes()).ok()?).ok()?;
        Some(MetadataField { name, value })
    })
    .collect()
}

pub fn extract(bytes: &[u8], t: &Thresholds) -> Result<Document, ExtractError> {
    let oxide = PdfDocument::from_bytes(bytes.to_vec()).map_err(err)?;
    let lo = lopdf::Document::load_mem(bytes).map_err(err)?;
    let page_ids: Vec<lopdf::ObjectId> = lo.get_pages().into_values().collect();
    let (mut spans, mut lines, mut pages, mut signals) =
        (Vec::new(), Vec::new(), Vec::new(), SignalSet::default());
    for (index, page_id) in page_ids.iter().enumerate() {
        let number = index as u32 + 1;
        let (mx0, my0, mx1, my1) = oxide.get_page_media_box(index).map_err(err)?;
        let media = (mx0, my0, mx1, my1);
        let scan = content::scan(&lo, *page_id).map_err(err)?;
        let fonts = content::fonts(&lo, *page_id).map_err(err)?;
        glyphs::missing_to_unicode(&scan, &fonts, number, &mut signals);
        let chars = oxide.extract_chars(index).map_err(err)?;
        let first = spans.len();
        for run in runs::assemble(&chars, &scan.shows, t.baseline_tolerance_pt) {
            let id = SpanId(spans.len() as u32);
            let span = Span {
                id,
                text: run.text,
                page: number,
                bbox: Some(run.bbox),
                font_size: Some(run.font_size),
                color: Some(run.color),
                region: Region::Body,
            };
            if off_page(&run.bbox, media) {
                signals.add(
                    SignalKind::HiddenText(HiddenReason::OffPage),
                    Some(number),
                    integrity::span_evidence(&span),
                );
            }
            spans.push(span);
        }
        let frames = frames::detect(&scan.segments);
        let laid = layout::layout_page(&mut spans[first..], number, my1 - my0, &frames, t);
        lines.extend(laid.lines);
        signals.extend(laid.signals);
        let span_count = (spans.len() - first) as u32;
        if span_count == 0 && scan.images > 0 {
            let evidence = Evidence {
                text: format!("{} image(s), no text", scan.images),
                span: None,
                bbox: None,
            };
            signals.add(SignalKind::ImageOnlyPage, Some(number), evidence);
        }
        pages.push(Page {
            number,
            size: Some((mx1 - mx0, my1 - my0)),
            image_count: scan.images,
            span_count,
        });
    }
    layout::running::confirm(&mut lines, &mut spans, t);
    signals.extend(integrity::span_signals(&spans, t));
    let metadata = metadata(&lo);
    signals.extend(integrity::metadata_signals(&metadata, &spans));
    let stored: Vec<SpanId> = spans.iter().map(|s| s.id).collect();
    let reading: Vec<SpanId> = lines.iter().flat_map(|l| l.spans.iter().copied()).collect();
    let displaced = order::displaced(&stored, &reading);
    if displaced >= t.min_displaced_spans {
        let evidence = Evidence {
            text: format!("{displaced} span(s) out of stored order"),
            span: None,
            bbox: None,
        };
        signals.add(
            SignalKind::ReadingOrderDisagreement {
                displaced_spans: displaced,
            },
            None,
            evidence,
        );
    }
    Ok(Document {
        format: Format::Pdf,
        pages,
        spans,
        lines,
        metadata,
        signals: signals.into_vec(),
    })
}
