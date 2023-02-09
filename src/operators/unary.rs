use std::fmt::Debug;

use crate::value::Value;

pub trait UnaryOperator<V: Value>: Debug + Clone {
    fn apply_1(&self, val: &V) -> V;
}
