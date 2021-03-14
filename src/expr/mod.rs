//! Expressions that can be evaluated into a [`Value`]

use crate::{
    value::{EntityMap, Value},
    Error,
};
use serde::Deserialize;
use std::collections::HashMap;

mod entity;
mod parse;

pub(crate) trait Environment {
    fn eval_func(&self, name: &str, args: &[Value]) -> Result<Value, Error>;
}

pub(crate) trait Evaluatable {
    fn eval(&self, ctx: &Context) -> Result<Value, Error>;

    fn eval_to_entity(&self, ctx: &Context) -> Result<EntityMap<Value>, Error> {
        match self.eval(ctx)? {
            Value::Entity(ent) => Ok(ent),
            unexpected => Err(Error::UnexpectedType {
                unexpected: unexpected.typename(),
                expected: "entity",
            }),
        }
    }
}

/// Context for evaluation
pub(crate) struct Context<'a> {
    #[allow(dead_code)]
    pub comp_lib: &'a dyn Environment,
}

impl<'a> Context<'a> {
    pub fn new(comp_lib: &'a dyn Environment) -> Self {
        Context { comp_lib }
    }

    pub fn call_function(&self, _name: &str, _args: &[Value]) -> Result<Value, Error> {
        unimplemented!()
    }
}

pub type EntityExpr = EntityMap<Expr>;

impl EntityExpr {
    pub(crate) fn eval(&self, ctx: &Context) -> Result<EntityMap<Value>, Error> {
        self.clone().try_map(|c| c.eval(ctx))
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct EntityExprBuilder<T>(EntityMap<T>);

#[cfg(test)]
impl<T> EntityExprBuilder<T> {
    pub fn new() -> Self {
        EntityExprBuilder(EntityMap::new())
    }

    pub fn add_component(mut self, name: &str, value: T) -> Self {
        self.0.add_component(name, value);
        self
    }

    pub fn build(self) -> EntityMap<T> {
        self.0
    }
}

//#[derive(PartialEq, Debug, Deserialize, Serialize)]
//#[serde(untagged)]
#[derive(PartialEq, Debug, Clone, Deserialize)]
#[serde(from = "parse::ParsedExprNode")]
pub enum Expr {
    KeyMap(HashMap<String, Expr>),
    Constant(Value),
    Vec(Vec<Expr>),
    Entity(EntityMap<Expr>),
    Func(String, Vec<Expr>),
}

impl Evaluatable for Expr {
    fn eval(&self, ctx: &Context) -> Result<Value, Error> {
        match self {
            Expr::KeyMap(m) => Ok(Value::KeyMap(
                m.iter()
                    .map(|(k, v)| Ok((k.clone(), v.eval(ctx)?)))
                    .collect::<Result<_, Error>>()?,
            )),
            Expr::Constant(v) => Ok(v.clone()),
            Expr::Vec(values) => Ok(Value::Vec(
                values
                    .iter()
                    .map(|v| v.eval(ctx))
                    .collect::<Result<_, Error>>()?,
            )),
            Expr::Entity(e) => Ok(Value::Entity(e.eval(ctx)?)),
            Expr::Func(func_name, args) => {
                let evaled_args: Vec<Value> = args
                    .iter()
                    .map(|arg| arg.eval(ctx))
                    .collect::<Result<_, Error>>()?;
                ctx.call_function(func_name, &evaled_args)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expr::{EntityExpr, Expr},
        value::Value,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestComp {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_parse() {
        let entity_json = r#"
            {
                "test_comp": {
                    "x": 42.0,
                    "y": 42.0
                }
            }
        "#;

        let expected_test_comp_expr = Expr::KeyMap(
            vec![
                ("x".to_string(), Expr::Constant(Value::F32(42.0))),
                ("y".to_string(), Expr::Constant(Value::F32(42.0))),
            ]
            .into_iter()
            .collect(),
        );

        let parsed_entity: EntityExpr = serde_json::from_str(entity_json).unwrap();

        let components: Vec<(_, _)> = parsed_entity.components().collect();

        assert_eq!(components, vec![("test_comp", &expected_test_comp_expr)]);
    }
}
