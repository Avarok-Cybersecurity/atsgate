//! JSON Schemas generated from the Rust types, so the schema files and the code cannot drift.

pub fn all() -> Result<Vec<(String, Vec<u8>)>, String> {
    let render = |schema: schemars::Schema| -> Result<Vec<u8>, String> {
        let mut text = serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())?;
        text.push('\n');
        Ok(text.into_bytes())
    };
    Ok(vec![
        (
            "schemas/career.schema.json".to_owned(),
            render(schemars::schema_for!(atsgate_career::Career))?,
        ),
        (
            "schemas/blueprint.schema.json".to_owned(),
            render(schemars::schema_for!(atsgate_blueprint::Blueprint))?,
        ),
        (
            "schemas/tailoring.schema.json".to_owned(),
            render(schemars::schema_for!(atsgate_career::Tailoring))?,
        ),
        (
            "schemas/contact.schema.json".to_owned(),
            render(schemars::schema_for!(atsgate_career::PrivateContact))?,
        ),
        (
            "schemas/report.schema.json".to_owned(),
            render(schemars::schema_for!(atsgate_score::ScoreOutput))?,
        ),
    ])
}
