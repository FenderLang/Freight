use crate::{
    error::FreightError, execution_engine::ExecutionEngine, function::FunctionRef, TypeSystem,
};

use std::{fmt::Debug, ops::Deref};

pub struct NativeFunction<TS: TypeSystem>(
    fn(&mut ExecutionEngine<TS>, Vec<TS::Value>) -> Result<TS::Value, FreightError>,
);

impl<TS: TypeSystem> NativeFunction<TS> {
    pub fn new(
        value: fn(&mut ExecutionEngine<TS>, Vec<TS::Value>) -> Result<TS::Value, FreightError>,
    ) -> Self {
        Self(value)
    }
}

impl<TS: TypeSystem> Deref for NativeFunction<TS> {
    type Target = fn(&mut ExecutionEngine<TS>, Vec<TS::Value>) -> Result<TS::Value, FreightError>;

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

#[derive(Debug)]
pub enum Expression<TS: TypeSystem> {
    RawValue(TS::Value),
    Variable(VariableType),
    Global(usize),
    BinaryOpEval(TS::BinaryOp, Box<[Expression<TS>; 2]>),
    UnaryOpEval(TS::UnaryOp, Box<Expression<TS>>),
    StaticFunctionCall(FunctionRef<TS>, Vec<Expression<TS>>),
    DynamicFunctionCall(Box<Expression<TS>>, Vec<Expression<TS>>),
    NativeFunctionCall(NativeFunction<TS>, Vec<Expression<TS>>),
    FunctionCapture(FunctionRef<TS>),
    AssignStack(usize, Box<Expression<TS>>),
    AssignGlobal(usize, Box<Expression<TS>>),
    AssignDynamic(Box<[Expression<TS>; 2]>),
}

impl<TS: TypeSystem> Expression<TS> {
    pub fn stack(addr: usize) -> Expression<TS> {
        Expression::Variable(VariableType::Stack(addr))
    }

    pub fn captured(addr: usize) -> Expression<TS> {
        Expression::Variable(VariableType::Captured(addr))
    }
}