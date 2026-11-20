//! Lookup over the shipped data: canonical names, curated and imported aliases, broader and
//! narrower links, titles, and the data headers.

use atsgate_taxonomy::{MatchKind, SourceName, Taxonomy};

fn taxonomy() -> &'static Taxonomy {
    Taxonomy::builtin().expect("built-in taxonomy loads")
}

fn skill(text: &str) -> (String, String, MatchKind) {
    let r = taxonomy()
        .resolve_skill(text)
        .unwrap_or_else(|| panic!("{text:?} resolves"));
    (r.canonical, r.taxonomy_ref, r.match_kind)
}

#[test]
fn curated_aliases_resolve_to_one_canonical_concept() {
    for (alias, canonical, reference) in [
        ("JS", "JavaScript", "onet-tech:JavaScript"),
        ("k8s", "Kubernetes", "onet-tech:Kubernetes"),
        ("Postgres", "PostgreSQL", "onet-tech:PostgreSQL"),
        ("Rust lang", "Rust", "onet-tech:Rust programming language"),
        ("golang", "Go", "onet-tech:Go"),
        (
            "Amazon Web Services",
            "AWS",
            "onet-tech:Amazon Web Services AWS software",
        ),
    ] {
        assert_eq!(
            skill(alias),
            (canonical.into(), reference.into(), MatchKind::Alias),
            "{alias}"
        );
    }
}

#[test]
fn canonical_names_match_exactly_after_folding() {
    assert_eq!(skill("  kubernetes ").2, MatchKind::Exact);
    assert_eq!(skill("POSTGRESQL").0, "PostgreSQL");
    assert_eq!(skill("Rust").1, "onet-tech:Rust programming language");
}

#[test]
fn imported_labels_and_aliases_resolve() {
    let (canonical, reference, kind) = skill("Python (computer programming)");
    assert_eq!(canonical, "Python (computer programming)");
    assert_eq!(kind, MatchKind::Exact);
    assert!(reference.starts_with("http://data.europa.eu/esco/skill/"));
    assert_eq!(
        skill("Server-side JavaScript").2,
        MatchKind::Alias,
        "an ESCO alternative label"
    );
    let (_, reference, _) = skill("manage ICT virtualisation environments");
    assert!(reference.starts_with("http://data.europa.eu/esco/skill/"));
}

#[test]
fn unknown_text_does_not_resolve() {
    assert!(taxonomy().resolve_skill("zzqx frobnication").is_none());
    assert!(taxonomy().resolve_skill("").is_none());
}

#[test]
fn broader_and_narrower_links_relate_concepts() {
    let t = taxonomy();
    let rust = t.resolve_skill("Rust").expect("Rust").concept;
    let broader = t
        .relate("Development environment software", rust)
        .expect("category relates");
    assert_eq!(broader.match_kind, MatchKind::Broader);
    let category = t
        .resolve_skill("Development environment software")
        .expect("category")
        .concept;
    assert_eq!(
        t.relate("Rust lang", category).map(|r| r.match_kind),
        Some(MatchKind::Narrower)
    );
    assert_eq!(
        t.relate("rustlang", rust).map(|r| r.match_kind),
        Some(MatchKind::Alias)
    );
    assert!(t.relate("Microsoft Excel", rust).is_none());
}

#[test]
fn same_name_across_sources_counts_as_the_same_concept() {
    let t = taxonomy();
    let esco_js = t
        .concepts()
        .find(|c| c.source() == SourceName::Esco && c.canonical() == "JavaScript")
        .expect("ESCO has JavaScript");
    assert_eq!(
        t.relate("JavaScript", esco_js.id()).map(|r| r.match_kind),
        Some(MatchKind::Exact)
    );
}

#[test]
fn titles_resolve_to_occupations() {
    let t = taxonomy();
    let r = t.resolve_title("Software Developers").expect("O*NET title");
    assert_eq!(r.taxonomy_ref, "onet-soc:15-1252.00");
    assert_eq!(r.match_kind, MatchKind::Exact);
    let alias = t.resolve_title("Paralegal").expect("paralegal");
    assert_eq!(alias.taxonomy_ref, "onet-soc:23-2011.00");
    let esco = t
        .resolve_title_from("paralegal", SourceName::Esco)
        .expect("ESCO occupation");
    assert!(
        esco.taxonomy_ref
            .starts_with("http://data.europa.eu/esco/occupation/")
    );
    assert!(t.resolve_skill("Software Developers").is_none());
}

#[test]
fn versions_and_credits_come_from_the_data_headers() {
    let t = taxonomy();
    assert_eq!(t.versions().get("onet").map(String::as_str), Some("31.0"));
    assert_eq!(t.versions().get("esco").map(String::as_str), Some("v1.2.1"));
    assert!(
        t.credits()
            .iter()
            .any(|c| c.contains("O*NET 31.0 Database"))
    );
    assert!(
        t.credits()
            .iter()
            .any(|c| c.contains("ESCO classification"))
    );
}

#[test]
fn no_lightcast_data_is_loaded() {
    assert!(
        taxonomy()
            .concepts()
            .all(|c| !c.taxonomy_ref().to_lowercase().contains("lightcast"))
    );
}

#[test]
fn backend_engineer_titles_resolve_to_software_developers() {
    for title in ["Backend Engineer", "Back-end Engineer", "back end engineer"] {
        let r = taxonomy()
            .resolve_title_from(title, SourceName::Onet)
            .unwrap_or_else(|| panic!("{title:?} resolves"));
        assert_eq!(
            (r.canonical.as_str(), r.taxonomy_ref.as_str()),
            ("Software Developers", "onet-soc:15-1252.00"),
            "{title}"
        );
    }
}

#[test]
fn a_contextual_title_resolves_only_inside_its_group() {
    let t = taxonomy();
    let in_software = t
        .contextual_title("Staff Engineer", "15-1252.00")
        .map(|r| r.taxonomy_ref);
    assert_eq!(
        in_software, None,
        "seniority words are the caller's to strip"
    );
    let engineer = t
        .contextual_title("Engineer", "15-1252.00")
        .expect("software");
    assert_eq!(engineer.taxonomy_ref, "onet-soc:15-1252.00");
    let security = t
        .contextual_title("Security Engineer", "15-1299.07")
        .expect("15-12");
    assert_eq!(security.taxonomy_ref, "onet-soc:15-1299.05");
    assert!(t.contextual_title("Engineer", "17-2051.00").is_none());
}

#[test]
fn title_candidates_list_every_occupation_in_rank_order() {
    let refs: Vec<String> = taxonomy()
        .title_candidates("Systems Engineer", SourceName::Onet)
        .into_iter()
        .map(|r| r.taxonomy_ref)
        .collect();
    assert_eq!(
        refs.first().map(String::as_str),
        Some("onet-soc:13-1081.01")
    );
    assert!(refs.contains(&"onet-soc:15-1252.00".to_owned()));
    assert_eq!(
        taxonomy()
            .resolve_title_from("Systems Engineer", SourceName::Onet)
            .map(|r| r.taxonomy_ref),
        refs.first().cloned()
    );
}
