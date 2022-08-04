//! Prints a blueprint system for entity components systems

use bevy::reflect::TypeUuid;
use expr::Evaluatable;

use std::ffi::OsStr;
use tracing::info;

pub mod bevy_prints;
pub mod expr;
mod runtime;
pub mod value;

use crate::{
    expr::{EntityExpr, Expr},
    value::EntityMap,
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Errors from loading and executing [`Blueprint`]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Unexpected type {unexpected}, expected {expected}")]
    UnexpectedType {
        unexpected: &'static str,
        expected: &'static str,
    },
    #[error("Could not determine entity name from path {0:?}")]
    CouldNotDetermineEntityName(PathBuf),
    #[error("Error loading {0:?}: {1}")]
    LoadError(PathBuf, #[source] std::io::Error),
    #[error("Error parsing {0:?}: {1}")]
    ParseError(String, anyhow::Error),
    #[error("Error creating component")]
    ToComponentError(#[from] value::ToComponentError),
    #[error("Function `{0}` not defined")]
    UndefinedFunctionError(String),
    #[error("Unknown component `{0}`")]
    UnknownComponent(String),
}

/// Blueprint for creating an entity.
#[derive(Debug, TypeUuid)]
#[uuid = "1e71b7ab-6867-4711-8ac7-51538edf2403"]
pub struct Blueprint {
    #[allow(dead_code)]
    name: String,
    expr: Expr,
}

impl Evaluatable for Blueprint {
    fn eval(&self, ctx: &expr::Context) -> Result<value::Value, Error> {
        self.expr.eval(ctx)
    }
}

impl Blueprint {
    pub fn new(name: impl Into<String>, entity: EntityExpr) -> Self {
        Blueprint {
            name: name.into(),
            expr: Expr::Entity(entity),
        }
    }

    /// Load blueprint from a json file
    pub fn load_from_json(filename: &Path) -> Result<Blueprint, Error> {
        let blueprint_file =
            File::open(filename).map_err(|e| Error::LoadError(filename.to_owned(), e))?;

        let name = filename
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| Error::CouldNotDetermineEntityName(filename.to_owned()))?;

        let expr: Expr = serde_json::from_reader(blueprint_file).map_err(|e| {
            Error::ParseError(
                filename.to_string_lossy().to_string(),
                anyhow::Error::new(e),
            )
        })?;

        Ok(Blueprint {
            name: name.to_string(),
            expr,
        })
    }

    pub fn load_from_ron_bytes(filename: &Path, data: &[u8]) -> Result<Blueprint, Error> {
        let name = filename
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| Error::CouldNotDetermineEntityName(filename.to_owned()))?;

        let expr: EntityMap<Expr> = ron::de::from_bytes(data).map_err(|e| {
            Error::ParseError(
                filename.to_string_lossy().to_string(),
                anyhow::Error::new(e),
            )
        })?;

        info!(filename=?filename, "Loaded blueprint");

        info!(blueprint_data=?expr, "blueprint data");

        Ok(Blueprint {
            name: name.to_string(),
            expr: Expr::Entity(expr),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Blueprint;

    #[test]
    fn test_test_blueprint_parses() {
        Blueprint::load_from_ron_bytes(
            Path::new("../assets/blueprints/test.bp.ron"),
            include_bytes!("../assets/blueprints/test.bp.ron")
        ).expect("assets/blueprints/test.bp.ron parses");
    }

    #[test]
    fn test_example_blueprint_parses() {
        Blueprint::load_from_ron_bytes(
            Path::new("../assets/blueprints/example.bp.ron"),
            include_bytes!("../assets/blueprints/example.bp.ron")
        ).expect("assets/blueprints/example.bp.ron parses");
    }
}