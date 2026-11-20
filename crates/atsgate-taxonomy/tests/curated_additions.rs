//! Curated concepts the imported taxonomies lack, aliases they miss, and collisions corrected.

use atsgate_taxonomy::Taxonomy;

fn resolves(text: &str) -> Option<(String, String)> {
    let r = Taxonomy::builtin().expect("taxonomy").resolve_skill(text)?;
    Some((r.canonical, r.taxonomy_ref))
}

fn canonical(text: &str) -> Option<String> {
    resolves(text).map(|(c, _)| c)
}

#[test]
fn curated_concepts_for_common_engineering_terms_resolve() {
    for (text, want) in [
        ("Svelte", "Svelte"),
        ("Axum", "Axum"),
        ("WebSocket", "WebSocket"),
        ("WebSockets", "WebSocket"),
        ("Protocol Buffers", "Protocol Buffers"),
        ("protobuf", "Protocol Buffers"),
        ("Tailwind CSS", "Tailwind CSS"),
        ("Tailwind", "Tailwind CSS"),
        ("Model Context Protocol", "Model Context Protocol"),
        ("MCP", "Model Context Protocol"),
        ("change data capture", "change data capture"),
        ("CDC", "change data capture"),
        ("stream processing", "stream processing"),
        ("infrastructure as code", "infrastructure as code"),
        ("IaC", "infrastructure as code"),
        ("local-first software", "local-first software"),
        ("local-first", "local-first software"),
        ("microservices", "microservices"),
        ("microservice architecture", "microservices"),
        ("fault tolerance", "fault tolerance"),
    ] {
        assert_eq!(canonical(text).as_deref(), Some(want), "{text}");
    }
}

#[test]
fn engineering_phrasings_reach_the_esco_concepts() {
    for (text, want) in [
        ("distributed systems", "distributed computing"),
        ("concurrency", "use concurrent programming"),
        ("multithreading", "use concurrent programming"),
        ("async", "use concurrent programming"),
    ] {
        assert_eq!(canonical(text).as_deref(), Some(want), "{text}");
    }
}

#[test]
fn colliding_aliases_resolve_to_the_right_concept() {
    assert_ne!(canonical("data collection").as_deref(), Some("statistics"));
    assert_ne!(canonical("big data").as_deref(), Some("statistics"));
    assert_eq!(canonical("ICT security").as_deref(), Some("cyber security"));
    assert_eq!(
        resolves("SaaS").map(|(_, r)| r).as_deref(),
        Some("onet-tech:Software as a service SaaS")
    );
    assert_eq!(
        canonical("Software as a service").as_deref(),
        Some("Software as a service SaaS")
    );
    assert_eq!(canonical("TLS").as_deref(), Some("Transport Layer Security"));
    assert_eq!(
        canonical("transport layer security").as_deref(),
        Some("Transport Layer Security")
    );
}
