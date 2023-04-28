use operators::{BinaryOperator, Initializer, UnaryOperator};
use std::fmt::Debug;
use value::Value;

pub mod error;
pub mod execution_engine;
pub mod expression;
pub mod function;
pub mod operators;
pub mod ref_pool;
pub mod slice_pool;
pub mod value;

/// Defines the type system for a programming language
pub trait TypeSystem: Debug + Clone + 'static {
    /// The value type for a language
    type Value: Value<TS = Self>;
    /// The unary operator type for a language
    type UnaryOp: UnaryOperator<Self::Value>;
    /// The binary operator type for a language
    type BinaryOp: BinaryOperator<Self::Value>;
    /// The initializers type for creating new values that take multiple expressions
    type Init: Initializer<Self>;
    /// The type id type for a language
    type TypeId: PartialEq + Debug;
    /// A global context object to be stored in the ExecutionEngine
    type GlobalContext: Debug;
}

#[cfg(test)]
mod tests;
