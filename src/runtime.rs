#![allow(dead_code)]

use crate::{expr::Environment, value::Value, Error};
use std::collections::HashMap;

struct Function {
    func_impl: Box<dyn Fn(&[Value]) -> Value>,
}

impl Function {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&[Value]) -> Value + 'static,
    {
        Function {
            func_impl: Box::new(f),
        }
    }

    fn eval(&self, args: &[Value]) -> Result<Value, Error> {
        Ok((self.func_impl)(args))
    }
}

pub struct SimpleRuntime {
    functions: HashMap<String, Function>,
}

impl SimpleRuntime {
    pub fn new() -> Self {
        SimpleRuntime {
            functions: HashMap::new(),
        }
    }

    pub fn register_func<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[Value]) -> Value + 'static,
    {
        self.functions.insert(name.to_string(), Function::new(f));
    }
}

impl Environment for SimpleRuntime {
    fn eval_func(&self, name: &str, args: &[Value]) -> Result<Value, Error> {
        let func = self
            .functions
            .get(name)
            .ok_or_else(|| Error::UndefinedFunctionError(name.to_string()))?;
        func.eval(args)
    }
}
