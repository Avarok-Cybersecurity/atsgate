//! The gazetteer's two scopes: postings (what a generator may call a requirement) and resumes
//! (what a scorer may credit against a skill the blueprint already names).

use atsgate_taxonomy::{Gazetteer, Scope, Taxonomy};

fn found(scope: Scope, text: &str) -> Vec<String> {
    let t = Taxonomy::builtin().expect("taxonomy");
    let gz = Gazetteer::new(t, scope);
    gz.find(text)
        .into_iter()
        .map(|m| t.concept(m.concept).canonical().to_owned())
        .collect()
}

/// Phrases from real postings that ESCO alternative labels or one-word labels turned into skills.
const NOISE: &str = "Send us an email. Help us clean the data infrastructure while you enjoy \
    working with curiosity. You will detect fraud with radar and prediction, collect statistics, \
    replace circuit breakers and filters, and Lead the transport team.";

#[test]
fn postings_ignore_esco_alternative_labels_and_single_words() {
    assert_eq!(found(Scope::Postings, NOISE), Vec::<String>::new());
    // A resume is matched only against skills a blueprint already names, so it keeps them.
    assert!(found(Scope::Resumes, NOISE).contains(&"statistics".to_owned()));
}

#[test]
fn postings_drop_esco_transversal_and_attitude_labels() {
    let text = "You show commitment, demonstrate curiosity and perform cleaning duties.";
    assert_eq!(found(Scope::Postings, text), Vec::<String>::new());
}

#[test]
fn postings_keep_technical_labels_curated_aliases_and_the_allow_list() {
    let text = "Experience with machine learning, cloud technologies, DevOps, GDPR, ICT security \
        and Rust.";
    assert_eq!(
        found(Scope::Postings, text),
        [
            "machine learning",
            "cloud technologies",
            "DevOps",
            "GDPR",
            "cyber security",
            "Rust"
        ]
    );
}
