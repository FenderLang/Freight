use crate::{
    collection_pool::PooledVec, error::FreightError, execution_engine::ExecutionEngine,
    function::FunctionRef, TypeSystem,
};

use std::{fmt::Debug, ops::Deref};

type NativeFuncInnerAlias<TS> = fn(
    &mut ExecutionEngine<TS>,
    PooledVec<<TS as TypeSystem>::Value>,
) -> Result<<TS as TypeSystem>::Value, FreightError>;

#[derive(Clone)]
pub struct NativeFunction<TS: TypeSystem>(NativeFuncInnerAlias<TS>);

impl<TS: TypeSystem> NativeFunction<TS> {
    pub fn new(value: NativeFuncInnerAlias<TS>) -> Self {
        Self(value)
    }
}

impl<TS: TypeSystem> Deref for NativeFunction<TS> {
    type Target = NativeFuncInnerAlias<TS>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<TS: TypeSystem> Debug for NativeFunction<TS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NativeFunction").finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableType {
    Captured(usize),
    Stack(usize),
    Global(usize),
}

/// Represents an expression tree that can be evaluated via an [ExecutionEngine]
#[derive(Debug)]
pub enum Expression<TS: TypeSystem> {
    /// Evaluate to a raw value, no computation required
    RawValue(TS::Value),
    /// Retrieve a variable value as a reference
    Variable(VariableType),
    /// Evaluate a binary operation on two sub-expressions
    BinaryOpEval(TS::BinaryOp, Box<[Expression<TS>; 2]>),
    /// Evaluate a unary operation on a sub-expression
    UnaryOpEval(TS::UnaryOp, Box<Expression<TS>>),
    Initialize(TS::Init, Vec<Expression<TS>>),

    /// Invoke a function that is known at compiletime
    StaticFunctionCall(FunctionRef<TS>, Vec<Expression<TS>>),
    /// Invoke a function whose identity is not known until runtime
    DynamicFunctionCall(Box<Expression<TS>>, Vec<Expression<TS>>),
    /// Invoke a native function
    NativeFunctionCall(NativeFunction<TS>, Vec<Expression<TS>>),
    /// Capture values from an environment, for closures
    FunctionCapture(FunctionRef<TS>),
    /// Assign a value on the stack
    AssignStack(usize, Box<Expression<TS>>),
    /// Assign a global value
    AssignGlobal(usize, Box<Expression<TS>>),
    /// Assign to a reference that will not be determined until runtime
    AssignDynamic(Box<[Expression<TS>; 2]>),
    /// An expression which can be returned to
    ReturnTarget(usize, Box<Expression<TS>>),
    /// Return to the specified return target
    Return(usize, Box<Expression<TS>>),
}

impl<TS: TypeSystem> Expression<TS> {
    /// Shorthand for a stack variable
    pub fn stack(addr: usize) -> Expression<TS> {
        Expression::Variable(VariableType::Stack(addr))
    }

    /// Shorthand for a captured variable
    pub fn captured(addr: usize) -> Expression<TS> {
        Expression::Variable(VariableType::Captured(addr))
    }

    /// Shorthand for a global variable
    pub fn global(addr: usize) -> Expression<TS> {
        Expression::Variable(VariableType::Global(addr))
    }
}
