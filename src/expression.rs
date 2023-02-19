use crate::{
    function::{FunctionRef, InvokeNative},
    TypeSystem,
};

use std::fmt::Debug;

pub struct NativeFunction<TS: TypeSystem>(pub(crate) Box<dyn InvokeNative<TS>>);

#[derive(Debug)]
pub enum Expression<TS: TypeSystem> {
    RawValue(TS::Value),
    Variable(usize),
    Global(usize),
    BinaryOpEval(TS::BinaryOp, Box<[Expression<TS>; 2]>),
    UnaryOpEval(TS::UnaryOp, Box<Expression<TS>>),
    StaticFunctionCall(FunctionRef<TS>, Vec<Expression<TS>>),
    DynamicFunctionCall(Box<Expression<TS>>, Vec<Expression<TS>>),
    NativeFunctionCall(NativeFunction<TS>, Vec<Expression<TS>>),
    FunctionCapture(FunctionRef<TS>),
    AssignStack(usize, Box<Expression<TS>>),
    AssignGlobal(usize, Box<Expression<TS>>),
}

impl<TS: TypeSystem> Debug for NativeFunction<TS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NativeFunction").finish()
    }
}