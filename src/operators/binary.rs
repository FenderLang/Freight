use std::fmt::Debug;

use crate::value::Value;

pub trait BinaryOperator<V: Value>: Debug + Clone {
    fn apply_2(&self, a: &V, b: &V) -> V;
}
