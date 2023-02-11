use operators::{binary::BinaryOperator, unary::UnaryOperator};
use value::Value;

pub mod execution_context;
pub mod expression_builder;
pub mod function;
pub mod instruction;
pub mod value;
pub mod vm_writer;
pub mod operators;

pub trait TypeSystem {
    type Value: Value;
    type UnaryOp: UnaryOperator<Self::Value>;
    type BinaryOp: BinaryOperator<Self::Value>;
    type TypeId;
}
