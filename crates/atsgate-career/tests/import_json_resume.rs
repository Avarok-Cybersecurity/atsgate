//! Plain JSON Resume imports by adding ids and an empty evidence block; nothing else changes,
//! and fields JSON Resume does not define are errors.

use atsgate_career::{ImportError, ImportFormat, import};
use serde_json::{Value, json};

fn resume() -> Value {
    json!({
        "basics": { "name": "Jordan Example", "label": "Engineer", "email": "jordan@example.com" },
        "work": [
            { "name": "Northwind", "position": "Engineer", "startDate": "2020-01", "highlights": ["Built it."] },
            { "name": "Northwind", "position": "Senior Engineer", "startDate": "2022-01" }
        ],
        "education": [{ "institution": "State University", "studyType": "BS", "area": "Physics", "endDate": "2016" }],
        "skills": [{ "name": "Languages", "keywords": ["Rust"] }]
    })
}

fn run(doc: &Value) -> Result<atsgate_career::Career, ImportError> {
    import(&doc.to_string(), ImportFormat::JsonResume)
}

#[test]
fn entries_get_unique_ids_and_the_rest_is_unchanged() {
    let career = run(&resume()).expect("imports");
    let ids: Vec<String> = career
        .work
        .iter()
        .flatten()
        .map(|w| w.extension.as_ref().expect("ext").id.as_str().to_owned())
        .collect();
    assert_eq!(ids, ["northwind", "northwind-2"]);
    let school = &career.education.as_ref().expect("education")[0];
    assert_eq!(
        school.extension.as_ref().expect("ext").id.as_str(),
        "state-university"
    );
    let mut back: Value = serde_json::from_str(&career.to_json().expect("json")).expect("json");
    let obj = back.as_object_mut().expect("object");
    obj.remove("x-atsgate");
    for section in ["work", "education"] {
        for entry in obj[section].as_array_mut().expect("array") {
            entry.as_object_mut().expect("entry").remove("x-atsgate");
        }
    }
    assert_eq!(back, resume(), "only x-atsgate blocks were added");
}

#[test]
fn unknown_fields_and_existing_career_files_are_rejected() {
    let mut extra = resume();
    extra["work"][0]["company"] = json!("Northwind");
    assert!(matches!(run(&extra), Err(ImportError::Career(_))));
    let mut career = resume();
    career["x-atsgate"] = json!({});
    assert!(matches!(run(&career), Err(ImportError::AlreadyCareer)));
    assert!(matches!(
        import("[1]", ImportFormat::JsonResume),
        Err(ImportError::Unrepresentable { .. })
    ));
    assert!(matches!(
        import("{", ImportFormat::JsonResume),
        Err(ImportError::Parse { .. })
    ));
}
