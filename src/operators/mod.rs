pub mod binary;
pub mod unary;

#[derive(Clone, Debug)]
pub enum Operator<TS: crate::TypeSystem> {
    Binary(TS::BinaryOp),
    Unary(TS::UnaryOp),
}
