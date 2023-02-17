use crate::{TypeSystem, function::FunctionRef, value::Value, operators::{unary::UnaryOperator, binary::BinaryOperator}};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestTypeSystem;

impl TypeSystem for TestTypeSystem {
    type Value = TestValueWrapper;

    type UnaryOp = TestUnaryOperator;

    type BinaryOp = TestBinaryOperator;

    type TypeId = TestTypeId;
}

#[derive(Debug, Clone)]
pub enum TestBinaryOperator {
    Add
}

#[derive(Debug, Clone)]
pub enum TestUnaryOperator {
    Inc
}

#[derive(PartialEq, Eq, Debug)]
pub enum TestTypeId {
    Number,
    Function,
    Null
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestValueWrapper(pub TestValue);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TestValue {
    Number(i64),
    Function(FunctionRef<TestTypeSystem>),
    #[default]
    Null,
}

impl Value for TestValueWrapper {
    type TS = TestTypeSystem;

    fn uninitialized_reference() -> Self {
        TestValueWrapper(TestValue::Null)
    }

    fn get_type(&self) -> &<Self::TS as TypeSystem>::TypeId {
        match &self.0 {
            TestValue::Number(_) => &TestTypeId::Number,
            TestValue::Function(_) => &TestTypeId::Function,
            TestValue::Null => &TestTypeId::Null,
        }
    }

    fn deep_clone(&self) -> Self {
        self.clone()
    }

    fn dupe_ref(&self) -> Self {
        self.clone()
    }

    fn cast_to_function(&self) -> Option<&FunctionRef<Self::TS>> {
        match self {
            Self(TestValue::Function(f)) => Some(f),
            _ => None
        }
    }
}

impl From<FunctionRef<TestTypeSystem>> for TestValueWrapper {
    fn from(value: FunctionRef<TestTypeSystem>) -> Self {
        TestValueWrapper(TestValue::Function(value))
    }
}

impl UnaryOperator<TestValueWrapper> for TestUnaryOperator {
    fn apply_1(&self, val: &TestValueWrapper) -> TestValueWrapper {
        match (self, &val.0) {
            (Self::Inc, TestValue::Number(n)) => TestValueWrapper(TestValue::Number(n + 1)),
            _ => panic!("Attempted to increment non-integer type"),
        }
    }
}

impl BinaryOperator<TestValueWrapper> for TestBinaryOperator {
    fn apply_2(&self, a: &TestValueWrapper, b: &TestValueWrapper) -> TestValueWrapper {
        match (self, &a.0, &b.0) {
            (Self::Add, TestValue::Number(a), TestValue::Number(b)) => TestValueWrapper(TestValue::Number(a + b)),
            _ => panic!("Attempt to add non-integer type"),
        }
    }

    fn priority(&self) -> usize {
        0
    }
}