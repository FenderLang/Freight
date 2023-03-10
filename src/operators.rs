use crate::value::Value;
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

pub trait Initializer<V: Value>: Debug + Clone {
    fn initialize(&self, values: Vec<V>) -> V;
}

impl<V: Value> Initializer<V> for () {
    fn initialize(&self, _: Vec<V>) -> V {
        V::default()
    }
}
