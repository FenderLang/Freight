use crate::{execution_engine::ExecutionEngine, value::Value};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Operator<TS: crate::TypeSystem> {
    Binary(TS::BinaryOp),
    Unary(TS::UnaryOp),
}

pub trait UnaryOperator<V: Value>: Debug + Clone {
    fn apply_1(&self, val: &V) -> V;
}

pub trait BinaryOperator<V: Value>: Debug + Clone {
    fn apply_2(&self, a: &V, b: &V) -> V;
}

pub trait Initializer<TS: crate::TypeSystem>: Debug + Clone {
    fn initialize(&self, values: Vec<TS::Value>, ctx: &mut ExecutionEngine<TS>) -> TS::Value;
}

impl<TS: crate::TypeSystem> Initializer<TS> for () {
    fn initialize(&self, _: Vec<TS::Value>, _: &mut ExecutionEngine<TS>) -> TS::Value {
        TS::Value::default()
    }
}
