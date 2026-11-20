//! The built-in blueprints, embedded from `blueprints/*.yaml` (`blueprint: builtin:<id>`).

use crate::{Blueprint, BlueprintError};

/// (id, YAML source). The id must equal the file's `id` field (a test checks it).
pub const BUILTINS: [(&str, &str); 10] = [
    (
        "software-engineer-backend",
        include_str!("../../../blueprints/software-engineer-backend.yaml"),
    ),
    (
        "software-engineer-systems-rust",
        include_str!("../../../blueprints/software-engineer-systems-rust.yaml"),
    ),
    (
        "ml-inference-engineer",
        include_str!("../../../blueprints/ml-inference-engineer.yaml"),
    ),
    (
        "software-engineer-frontend",
        include_str!("../../../blueprints/software-engineer-frontend.yaml"),
    ),
    (
        "data-scientist",
        include_str!("../../../blueprints/data-scientist.yaml"),
    ),
    (
        "product-designer",
        include_str!("../../../blueprints/product-designer.yaml"),
    ),
    (
        "marketing-manager",
        include_str!("../../../blueprints/marketing-manager.yaml"),
    ),
    (
        "sales-representative",
        include_str!("../../../blueprints/sales-representative.yaml"),
    ),
    (
        "accountant",
        include_str!("../../../blueprints/accountant.yaml"),
    ),
    (
        "paralegal",
        include_str!("../../../blueprints/paralegal.yaml"),
    ),
];

/// Parses the built-in blueprint `id`.
pub fn builtin(id: &str) -> Result<Blueprint, BlueprintError> {
    let (_, yaml) = BUILTINS
        .iter()
        .find(|(name, _)| *name == id)
        .ok_or_else(|| BlueprintError::UnknownBuiltin(id.to_owned()))?;
    Blueprint::from_yaml(yaml)
}
