//! Site partials, JSON-LD and marker injection, on the synthetic career.

use atsgate_career::{Career, PrivateContact, Tailoring};
use atsgate_render::{ContactProfile, PARTIALS, RenderError, document, inject, jsonld, partials};
use serde_json::Value;

fn fixture(path: &str) -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/");
    std::fs::read_to_string(format!("{root}{path}")).expect("fixture")
}

fn career() -> (Career, Tailoring) {
    let career = Career::from_json(&fixture("career/synthetic.career.json")).expect("career");
    let tailoring = Tailoring::from_yaml(&fixture("tailoring/synthetic.yaml")).expect("tailoring");
    (career, tailoring)
}

fn page() -> String {
    PARTIALS
        .iter()
        .map(|(name, _)| {
            format!("<section><!-- atsgate:{name} -->stale<!-- /atsgate:{name} --></section>\n")
        })
        .collect()
}

#[test]
fn partials_hold_the_selected_entries_in_order() {
    let (career, tailoring) = career();
    let doc = document(
        &tailoring.check(&career).expect("t"),
        &ContactProfile::Public,
    )
    .expect("doc");
    let parts = partials(&doc).expect("partials");
    let names: Vec<&str> = parts.keys().map(String::as_str).collect();
    assert_eq!(names, ["education", "experience", "projects", "skills"]);
    let experience = &parts["experience"];
    let northwind = experience
        .find("atsgate-experience-northwind")
        .expect("northwind");
    let contoso = experience
        .find("atsgate-experience-contoso")
        .expect("contoso");
    assert!(northwind < contoso);
    assert!(
        !experience.contains("fabrikam"),
        "an unselected role leaked"
    );
    assert!(experience.contains(r#"<time datetime="2020-01">Jan 2020</time> - Present"#));
    assert!(
        parts["projects"].contains(r#"<a href="https://ledgerline.example.com">Ledgerline</a>"#)
    );
    assert!(parts["skills"].starts_with("<dl class=\"atsgate-skills\">\n<dt>Data</dt>"));
}

#[test]
fn injection_fills_every_marker_and_leaves_the_rest_of_the_page() {
    let (career, tailoring) = career();
    let doc = document(
        &tailoring.check(&career).expect("t"),
        &ContactProfile::Public,
    )
    .expect("doc");
    let parts = partials(&doc).expect("partials");
    let filled = inject(&page(), &parts).expect("inject");
    assert!(!filled.contains("stale"));
    assert_eq!(filled.matches("<section>").count(), PARTIALS.len());
    let missing = page().replace("<!-- atsgate:skills -->", "");
    assert!(matches!(inject(&missing, &parts), Err(RenderError::Marker(name)) if name == "skills"));
}

#[test]
fn jsonld_describes_the_person_and_only_the_allowed_contact_details() {
    let (career, tailoring) = career();
    let tailored = tailoring.check(&career).expect("t");
    let public_doc = document(&tailored, &ContactProfile::Public).expect("doc");
    let public: Value = serde_json::from_str(
        &jsonld(&public_doc, &career.basics, &ContactProfile::Public).expect("ld"),
    )
    .expect("json");
    assert_eq!(public["@type"], "Person");
    assert_eq!(public["name"], "Jordan Example");
    assert_eq!(
        public["hasOccupation"][0]["name"],
        "Senior Software Engineer"
    );
    assert_eq!(public["worksFor"].as_array().map(Vec::len), Some(1));
    assert_eq!(public["worksFor"][0]["name"], "Northwind Analytics");
    assert_eq!(public["alumniOf"][0]["@type"], "EducationalOrganization");
    assert!(public.get("telephone").is_none());
    assert!(public["address"].get("streetAddress").is_none());

    let contact = PrivateContact::from_json(&fixture("contact/synthetic-private.json")).expect("c");
    let profile = ContactProfile::Private(contact);
    let private_doc = document(&tailored, &profile).expect("doc");
    let private: Value =
        serde_json::from_str(&jsonld(&private_doc, &career.basics, &profile).expect("ld"))
            .expect("json");
    assert_eq!(private["telephone"], "+1 555 0100");
    assert!(private["address"]["streetAddress"].is_string());
}
