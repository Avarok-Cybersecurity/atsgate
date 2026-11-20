//! Blueprint specs over a faked I/O seam: the only fake in ATSGate's tests. Parsing,
//! validation and generation are the real crates.

use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Files and URLs served from memory; fetches record the cap they were asked for.
#[derive(Default)]
struct Fake {
    files: BTreeMap<PathBuf, Vec<u8>>,
    urls: BTreeMap<String, Vec<u8>>,
    caps: std::cell::RefCell<Vec<u64>>,
}

impl Io for Fake {
    fn read(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{}: not found", path.display()))
    }
    fn fetch(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
        self.caps.borrow_mut().push(max_bytes);
        let body = self.urls.get(url).ok_or_else(|| format!("{url}: 404"))?;
        if body.len() as u64 > max_bytes {
            return Err(format!("{url}: body exceeds {max_bytes} bytes"));
        }
        Ok(body.clone())
    }
}

fn gazetteer() -> Gazetteer<'static> {
    Gazetteer::skills(atsgate_taxonomy::Taxonomy::builtin().expect("taxonomy"))
}

const BACKEND: &str = include_str!("../../../blueprints/software-engineer-backend.yaml");
const POSTING: &str =
    "Senior Backend Engineer\nRequirements\n- Go and PostgreSQL\nNice to have\n- Redis\n";

#[test]
fn specs_parse_into_their_four_forms() {
    assert_eq!(parse("builtin:accountant"), Ok(Spec::Builtin("accountant")));
    assert_eq!(
        parse("https://x.test/b.yaml"),
        Ok(Spec::Url("https://x.test/b.yaml"))
    );
    assert_eq!(parse("generate:jd.txt"), Ok(Spec::Generate("jd.txt")));
    assert_eq!(
        parse("blueprints/a.yaml"),
        Ok(Spec::File(Path::new("blueprints/a.yaml")))
    );
    assert!(
        parse("http://x.test/b.yaml").is_err(),
        "plain http is refused"
    );
    assert!(parse("generate:").is_err());
}

#[test]
fn builtin_file_and_url_blueprints_resolve_and_validate() {
    let mut io = Fake::default();
    io.files
        .insert("b.yaml".into(), BACKEND.as_bytes().to_vec());
    io.urls
        .insert("https://x.test/b.yaml".into(), BACKEND.as_bytes().to_vec());
    let gz = gazetteer();
    for spec in [
        "builtin:software-engineer-backend",
        "b.yaml",
        "https://x.test/b.yaml",
    ] {
        let r = resolve(spec, &io, &gz).unwrap_or_else(|e| panic!("{spec}: {e}"));
        assert_eq!(r.blueprint.id, "software-engineer-backend", "{spec}");
        assert!(r.generated_yaml.is_none());
    }
    assert_eq!(*io.caps.borrow(), [MAX_BLUEPRINT_BYTES]);
    assert!(resolve("builtin:nope", &io, &gz).is_err());
}

#[test]
fn a_url_blueprint_is_size_capped_and_schema_validated() {
    let mut io = Fake::default();
    let huge = vec![b'#'; MAX_BLUEPRINT_BYTES as usize + 1];
    io.urls.insert("https://x.test/huge.yaml".into(), huge);
    io.urls.insert(
        "https://x.test/bad.yaml".into(),
        BACKEND
            .replace("seniority: mid", "seniority: wizard")
            .into_bytes(),
    );
    io.urls.insert(
        "https://x.test/extra.json".into(),
        br#"{"id": "x", "surprise": 1}"#.to_vec(),
    );
    let gz = gazetteer();
    let err = resolve("https://x.test/huge.yaml", &io, &gz).expect_err("too large");
    assert!(err.contains("exceeds"), "{err}");
    assert!(
        resolve("https://x.test/bad.yaml", &io, &gz).is_err(),
        "unknown enum value"
    );
    assert!(
        resolve("https://x.test/extra.json", &io, &gz).is_err(),
        "unknown field"
    );
    assert!(resolve("https://x.test/missing.yaml", &io, &gz).is_err());
}

#[test]
fn generate_from_a_file_or_url_is_unreviewed_and_written_out() {
    let mut io = Fake::default();
    io.files
        .insert("jobs/Acme Backend.txt".into(), POSTING.as_bytes().to_vec());
    let html = "<html><body><h1>Senior Backend Engineer</h1><h2>Requirements</h2><p>Go and PostgreSQL</p></body></html>";
    io.urls
        .insert("https://jobs.test/42".into(), html.as_bytes().to_vec());
    let gz = gazetteer();
    let from_file = resolve("generate:jobs/Acme Backend.txt", &io, &gz).expect("generates");
    assert_eq!(from_file.blueprint.id, "acme-backend");
    assert!(!from_file.blueprint.provenance.reviewed);
    let yaml = from_file.generated_yaml.expect("yaml");
    assert_eq!(
        atsgate_blueprint::Blueprint::from_yaml(&yaml).expect("round trip"),
        from_file.blueprint
    );
    let from_url = resolve("generate:https://jobs.test/42", &io, &gz).expect("generates");
    assert_eq!(from_url.blueprint.id, "42");
    assert_eq!(from_url.blueprint.occupational_category, "15-1252.00");
    assert_eq!(*io.caps.borrow(), [MAX_POSTING_BYTES]);
}

#[test]
fn job_descriptions_are_sniffed_by_content() {
    assert_eq!(sniff(b"%PDF-1.7\n..."), Format::Pdf);
    assert_eq!(sniff(b"PK\x03\x04rest"), Format::Docx);
    assert_eq!(sniff(b"  <!doctype html><html>"), Format::Html);
    assert_eq!(sniff(b"Senior Engineer\nRust"), Format::Text);
    assert_eq!(
        id_from("https://x.test/jobs/ArtCraft_HN.html?ref=1"),
        Ok("artcraft-hn".to_owned())
    );
    assert!(id_from("https://x.test/???").is_err());
}
