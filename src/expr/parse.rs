use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::value::{EntityMap, Value};

use super::Expr;

#[derive(PartialEq, Debug, Serialize)]
struct ParsedStruct {
    name: String,
    members: HashMap<String, ParsedExprNode>,
}

// struct ParsedStructVisitor;

// impl <'de> Visitor<'de> for ParsedStructVisitor {
//     type Value = ParsedStruct;

//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(formatter, "struct")
//     }
// }

// impl <'de> Deserialize<'de> for ParsedStruct {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de> {
//         deserializer.deserialize_any(visitor)
//     }
// }

#[derive(PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ParsedExprNode {
    KeyMap(HashMap<String, ParsedExprNode>),
    String(String),
    I32(i32),
    F32(f32),
    Vec(Vec<ParsedExprNode>),
    Entity(EntityMap<ParsedExprNode>),
    Func(String, Vec<ParsedExprNode>),
}

impl From<ParsedExprNode> for Expr {
    fn from(parsed_node: ParsedExprNode) -> Self {
        match parsed_node {
            ParsedExprNode::KeyMap(m) => {
                Expr::KeyMap(m.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            ParsedExprNode::String(v) => Expr::Constant(Value::String(v)),
            ParsedExprNode::I32(v) => Expr::Constant(Value::I32(v)),
            ParsedExprNode::F32(v) => Expr::Constant(Value::F32(v)),
            ParsedExprNode::Vec(nodes) => Expr::Vec(nodes.into_iter().map(|n| n.into()).collect()),
            ParsedExprNode::Entity(entity_map) => Expr::Entity(entity_map.map(|c| c.into())),
            ParsedExprNode::Func(name, args) => {
                Expr::Func(name, args.into_iter().map(|n| n.into()).collect())
            }
        }
    }
}
