use std::fmt::Debug;

use crate::{execution_context::ExecutionContext, TypeSystem};

pub enum Instruction<TS: TypeSystem> {
    Create(usize, fn(&ExecutionContext<TS>) -> TS::Value),
    Move(usize, usize),
    MoveFromReturn(usize),
    MoveToReturn(usize),
    MoveRightOperand(usize),
    Invoke(usize, usize, usize),
    InvokeNative(fn(&mut ExecutionContext<TS>) -> TS::Value),
    Return(usize),
    ReturnConstant(TS::Value, usize),
    UnaryOperation(TS::UnaryOp),
    BinaryOperation(TS::BinaryOp),
    UnaryOperationWithHeld(TS::UnaryOp),
    BinaryOperationWithHeld(TS::BinaryOp),
    SetReturnRaw(TS::Value),
    SetRightOperandRaw(TS::Value),
    SetHeldRaw(TS::Value),
    PushRaw(TS::Value),
    Push(usize),
    Pop,
    PushFromReturn,
}

impl<TS: TypeSystem> Debug for Instruction<TS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(arg0, _arg1) => f.debug_tuple("Create").field(arg0).finish(),
            Self::Move(arg0, arg1) => f.debug_tuple("Move").field(arg0).field(arg1).finish(),
            Self::MoveFromReturn(arg0) => f.debug_tuple("MoveFromReturn").field(arg0).finish(),
            Self::MoveToReturn(arg0) => f.debug_tuple("MoveToReturn").field(arg0).finish(),
            Self::MoveRightOperand(arg0) => f.debug_tuple("MoveRightOperand").field(arg0).finish(),
            Self::Invoke(arg0, arg1, arg2) => f
                .debug_tuple("Invoke")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::InvokeNative(_) => f.debug_tuple("InvokeNative").finish(),
            Self::Return(stack_size) => f.debug_tuple("Return").field(stack_size).finish(),
            Self::ReturnConstant(arg0, stack_size) => f
                .debug_tuple("ReturnConstant")
                .field(arg0)
                .field(stack_size)
                .finish(),
            Self::UnaryOperation(arg0) => f.debug_tuple("UnaryOperation").field(arg0).finish(),
            Self::BinaryOperation(arg0) => f.debug_tuple("BinaryOperation").field(arg0).finish(),
            Self::UnaryOperationWithHeld(arg0) => {
                f.debug_tuple("UnaryOperationWithHeld").field(arg0).finish()
            }
            Self::BinaryOperationWithHeld(arg0) => f
                .debug_tuple("BinaryOperationWithHeld")
                .field(arg0)
                .finish(),
            Self::SetReturnRaw(arg0) => f.debug_tuple("SetReturnRaw").field(arg0).finish(),
            Self::SetRightOperandRaw(arg0) => {
                f.debug_tuple("SetRightOperandRaw").field(arg0).finish()
            }
            Self::SetHeldRaw(arg0) => f.debug_tuple("SetHeldRaw").field(arg0).finish(),
            Self::PushRaw(arg0) => f.debug_tuple("PushRaw").field(arg0).finish(),
            Self::Push(arg0) => f.debug_tuple("Push").field(arg0).finish(),
            Self::Pop => write!(f, "Pop"),
            Self::PushFromReturn => write!(f, "PushFromReturn"),
        }
    }
}
