use crate::Value;

#[derive(Clone, Default)]
enum FenderValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Error,
    #[default]
    Null,
}

enum BinaryOperator {
    Add,
    Sub,
    Div,
    Mod,
    Mul,
}

enum UnaryOperator {
    Neg,
    BoolNeg,
}

impl Value for FenderValue {}

impl crate::BinaryOperator<FenderValue> for BinaryOperator {
    fn apply(&self, a: &FenderValue, b: &FenderValue) -> FenderValue {
        use BinaryOperator::*;
        use FenderValue::*;
        match (self, a, b) {
            (Add, Int(a), Int(b)) => Int(a + b),
            (Add, Float(a), Float(b)) => Float(a + b),
            (Add, Int(a), Float(b)) => Float(*a as f64 + b),
            (Add, Float(a), Int(b)) => Float(a + *b as f64),
            (Add, _, _) => Error,

            (Sub, Int(a), Int(b)) => Int(a - b),
            (Sub, Float(a), Float(b)) => Float(a - b),
            (Sub, Int(a), Float(b)) => Float(*a as f64 - b),
            (Sub, Float(a), Int(b)) => Float(a - *b as f64),
            (Sub, _, _) => Error,

            (Mul, Int(a), Int(b)) => Int(a * b),
            (Mul, Float(a), Float(b)) => Float(a * b),
            (Mul, Int(a), Float(b)) => Float(*a as f64 * b),
            (Mul, Float(a), Int(b)) => Float(a * *b as f64),
            (Mul, _, _) => Error,

            (Div, Int(a), Int(b)) => Int(a / b),
            (Div, Float(a), Float(b)) => Float(a / b),
            (Div, Int(a), Float(b)) => Float(*a as f64 / b),
            (Div, Float(a), Int(b)) => Float(a / *b as f64),
            (Div, _, _) => Error,

            (Mod, Int(a), Int(b)) => Int(a % b),
            (Mod, Float(a), Float(b)) => Float(a % b),
            (Mod, Int(a), Float(b)) => Float(*a as f64 % b),
            (Mod, Float(a), Int(b)) => Float(a % *b as f64),
            (Mod, _, _) => Error,
        }
    }
}

impl crate::UnaryOperator<FenderValue> for UnaryOperator {
    fn apply(&self, val: &FenderValue) -> FenderValue {
        use FenderValue::*;
        use UnaryOperator::*;
        match (self, val) {
            (Neg, Int(val)) => Int(-val),
            (Neg, Float(val)) => Float(-val),
            (Neg, _) => Error,

            (BoolNeg, Bool(val)) => Bool(!val),
            (BoolNeg, _) => Error,
        }
    }
}
