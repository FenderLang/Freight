pub mod binary;
pub mod unary;

pub enum Operator<TS: crate::TypeSystem> {
    Binary(TS::BinaryOp),
    Unary(TS::UnaryOp),
}
