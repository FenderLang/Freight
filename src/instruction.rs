use crate::{
    execution_context::{ExecutionContext, Location},
    TypeSystem,
};
use std::fmt::Debug;

pub enum InstructionWrapper<TS: TypeSystem> {
    RawInstruction(Instruction<TS>),
    InstructionLocation(usize),
}

pub enum Instruction<TS: TypeSystem> {
    Create {
        location: Location,
        creation_callback: fn(&ExecutionContext<TS>) -> TS::Value,
    },
    SetRaw {
        location: Location,
        value: TS::Value,
    },

    Move {
        from: Location,
        to: Location,
    },

    PushRaw(TS::Value),
    Push(Location),
    Pop,

    UnaryOperation(TS::UnaryOp),
    BinaryOperation(TS::BinaryOp),
    UnaryOperationWithHeld(TS::UnaryOp),
    BinaryOperationWithHeld(TS::BinaryOp),

    Invoke(usize, usize, usize),
    InvokeDynamic(usize),
    InvokeNative(fn(&mut ExecutionContext<TS>) -> TS::Value),
    Return(usize),
    ReturnConstant(TS::Value, usize),
    CaptureValues,
}

impl<TS: TypeSystem> Debug for Instruction<TS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create { location, creation_callback:_ } => f.debug_struct("Create").field("location", location).finish(),
            Self::SetRaw { location, value } => f.debug_struct("SetRaw").field("location", location).field("value", value).finish(),
            Self::Move { from, to } => f.debug_struct("Move").field("from", from).field("to", to).finish(),
            Self::PushRaw(arg0) => f.debug_tuple("PushRaw").field(arg0).finish(),
            Self::Push(arg0) => f.debug_tuple("Push").field(arg0).finish(),
            Self::Pop => write!(f, "Pop"),
            Self::UnaryOperation(arg0) => f.debug_tuple("UnaryOperation").field(arg0).finish(),
            Self::BinaryOperation(arg0) => f.debug_tuple("BinaryOperation").field(arg0).finish(),
            Self::UnaryOperationWithHeld(arg0) => f.debug_tuple("UnaryOperationWithHeld").field(arg0).finish(),
            Self::BinaryOperationWithHeld(arg0) => f.debug_tuple("BinaryOperationWithHeld").field(arg0).finish(),
            Self::Invoke(arg0, arg1, arg2) => f.debug_tuple("Invoke").field(arg0).field(arg1).field(arg2).finish(),
            Self::InvokeDynamic(arg0) => f.debug_tuple("InvokeDynamic").field(arg0).finish(),
            Self::InvokeNative(_arg0) => f.debug_tuple("InvokeNative").finish(),
            Self::Return(arg0) => f.debug_tuple("Return").field(arg0).finish(),
            Self::ReturnConstant(arg0, arg1) => f.debug_tuple("ReturnConstant").field(arg0).field(arg1).finish(),
            Self::CaptureValues => write!(f, "CaptureValues"),
        }
    }
}
