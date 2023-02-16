use operators::{binary::BinaryOperator, unary::UnaryOperator};
use value::Value;

pub mod error;
pub mod execution_context;
pub mod expression;
pub mod function;
pub mod instruction;
pub mod operators;
pub mod value;
pub mod vm_writer;

pub trait TypeSystem: Clone {
    type Value: Value<TS = Self>;
    type UnaryOp: UnaryOperator<Self::Value>;
    type BinaryOp: BinaryOperator<Self::Value>;
    type TypeId;
}
