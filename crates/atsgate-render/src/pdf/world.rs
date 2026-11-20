//! Typst as a library: an in-memory [`World`] holding the template, its data and bundled fonts.
//! It never touches the file system, the network or the clock.

use std::sync::LazyLock;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

/// The template's path inside the virtual project; the data file sits next to it.
const MAIN_PATH: &str = "/resume.typ";
/// The path the template reads its data from with `json(...)`.
pub const DATA_PATH: &str = "/resume.json";

struct Fonts {
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

/// The only fonts the template may use: Libertinus Serif regular and bold (SIL OFL 1.1, see
/// `fonts/OFL.txt`). Embedded here rather than through `typst-assets`' `fonts` feature, which
/// would also embed fonts under other licenses (one of them GPL-3.0 with a font exception).
const FONT_FILES: [&[u8]; 2] = [
    include_bytes!("../../fonts/LibertinusSerif-Regular.otf"),
    include_bytes!("../../fonts/LibertinusSerif-Bold.otf"),
];

/// [`FONT_FILES`], parsed once per process.
static FONTS: LazyLock<Fonts> = LazyLock::new(|| {
    let fonts: Vec<Font> = FONT_FILES
        .into_iter()
        .flat_map(|data| Font::iter(Bytes::new(data)))
        .collect();
    Fonts {
        book: LazyHash::new(FontBook::from_fonts(&fonts)),
        fonts,
    }
});

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

fn file_id(path: &str) -> FileId {
    let vpath = VirtualPath::new(path).expect("the template paths are valid virtual paths");
    RootedPath::new(VirtualRoot::Project, vpath).intern()
}

pub struct ResumeWorld {
    main: Source,
    data_id: FileId,
    data: Bytes,
}

impl std::fmt::Debug for ResumeWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResumeWorld").finish_non_exhaustive()
    }
}

impl ResumeWorld {
    pub fn new(template: &str, data: Vec<u8>) -> Self {
        Self {
            main: Source::new(file_id(MAIN_PATH), template.to_owned()),
            data_id: file_id(DATA_PATH),
            data: Bytes::new(data),
        }
    }
}

impl World for ResumeWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &FONTS.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.data_id {
            Ok(self.data.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.fonts.get(index).cloned()
    }

    /// No clock: a template that asks for today's date fails to compile.
    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}
