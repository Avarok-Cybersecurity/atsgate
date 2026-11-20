//! The thomaspbraun.com patent to a JSON Resume publication, and its recognition to an award
//! whose `subject` is that publication.

use super::ImportError;
use super::ids::Ids;
use super::site_entries::{date, display, ext, source_ids};
use super::site_format::{Patent, PatentImage, Recognition};
use crate::item::{Image, ItemExtension};
use crate::resume::{Award, Publication};

pub(super) fn patent(p: Patent, ids: &mut Ids) -> Result<(Publication, Award), ImportError> {
    let Patent {
        title,
        number,
        url,
        assignee,
        granted,
        text,
        bullets,
        recognition,
        image,
        sources,
    } = p;
    let PatentImage { alt, w, h } = image;
    let Recognition {
        title: award,
        by,
        text: why,
    } = recognition;
    let id = ids.derive(&number, "patent")?;
    let image = Some(Image {
        alt,
        width: w,
        height: h,
    });
    let x = ItemExtension {
        display: display(None, None, image),
        number: Some(number),
        highlights: Some(bullets),
        ..ext(id.clone(), source_ids(sources, &title)?)
    };
    let publication = Publication {
        name: title,
        publisher: Some(assignee),
        release_date: Some(date("patent", granted)?),
        url: Some(url),
        summary: Some(text),
        extension: Some(x),
    };
    let award_x = ItemExtension {
        subject: Some(id),
        ..ext(ids.derive(&award, "award")?, Vec::new())
    };
    Ok((
        publication,
        Award {
            title: award,
            date: None,
            awarder: Some(by),
            summary: Some(why),
            extension: Some(award_x),
        },
    ))
}
