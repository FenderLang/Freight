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
    /// A function callback is used to generate a value that will be placed in `location`
    ///
    /// The `creation_callback` function will get called when the instruction is interpreted, by the `ExecutionContext`
    /// It is not called when instantiating this instruction.
    Create {
        location: Location,
        creation_callback: fn(&ExecutionContext<TS>) -> TS::Value,
    },

    /// Set the value of `Location` to be `value`
    SetRaw {
        location: Location,
        value: TS::Value,
    },

    /// Call the `Value::assign` function on the `value` at the given `location`.
    ///
    /// This is for implementations to handle assigning values themselves so they can handle things such as internal mutability.
    AssignRaw {
        location: Location,
        value: TS::Value,
    },

    /// Assign a value from one location to another
    ///
    /// This is for implementations to handle assigning values themselves so they can handle things such as internal mutability.
    Assign {
        from: Location,
        to: Location,
    },

    /// Move value from `from` to `to`.
    ///
    /// Moving from a register (`Location::Register`) will take the value from the register and leave the register with a null value.
    ///
    /// Moving from a stack location will always clone the value, leaving the original value intact.
    ///
    /// If you wish to move from a register without the value becoming null consider `Swap` or `Push`.
    Move {
        from: Location,
        to: Location,
    },

    /// Swap the the values of 2 locations.
    Swap(Location, Location),

    /// Pushes a value from a given location to the end of the stack
    Push(Location),

    /// Pushes a given value onto the end of the stack
    PushRaw(TS::Value),

    /// Pops the last value off of the stack moving it into the `Return` register
    ///
    /// To not have the `Return` register altered by the `Pop` instruction enable the `popped_register` feature.
    /// This will move the popped value into the `Popped` register instead.
    Pop,

    /// Uses the unary operation given on the value in the `Return` register, the result is left in the `Return` register
    UnaryOperation(TS::UnaryOp),

    /// Uses the given binary operation on the value in the `Return` and `RightOperand` registers, the result is left in the `Return` register
    BinaryOperation(TS::BinaryOp),

    /// Uses the unary operation given on the value in the `HELD_VALUE` stack location, the result is left in the `Return` register
    UnaryOperationWithHeld(TS::UnaryOp),

    /// Uses the given binary operation on the value in the `HELD_VALUE` stack location and `RightOperand` register, the result is left in the `Return` register.
    ///
    /// The `HELD_VALUE` is used as the first/left operand of the operation.
    BinaryOperationWithHeld(TS::BinaryOp),

    //TODO: @Redempt add comments for the remaining instructions
    Invoke {
        arg_count: usize,
        stack_size: usize,
        instruction: usize,
    },
    InvokeDynamic {
        arg_count: usize,
    },

    InvokeNative(fn(&mut ExecutionContext<TS>) -> TS::Value),
    Return {
        stack_size: usize,
    },
    ReturnConstant {
        value: TS::Value,
        stack_size: usize,
    },
    CaptureValues,
}

impl<TS: TypeSystem> Debug for Instruction<TS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create {
                location,
                creation_callback: _,
            } => f
                .debug_struct("Create")
                .field("location", location)
                .finish(),
            Self::SetRaw { location, value } => f
                .debug_struct("SetRaw")
                .field("location", location)
                .field("value", value)
                .finish(),
            Self::AssignRaw { location, value } => f
                .debug_struct("AssignRaw")
                .field("location", location)
                .field("value", value)
                .finish(),
            Self::Assign { from, to } => f
                .debug_struct("Assign")
                .field("from", from)
                .field("to", to)
                .finish(),
            Self::Move { from, to } => f
                .debug_struct("Move")
                .field("from", from)
                .field("to", to)
                .finish(),
            Self::Swap(arg0, arg1) => f.debug_tuple("Swap").field(arg0).field(arg1).finish(),
            Self::PushRaw(arg0) => f.debug_tuple("PushRaw").field(arg0).finish(),
            Self::Push(arg0) => f.debug_tuple("Push").field(arg0).finish(),
            Self::Pop => write!(f, "Pop"),
            Self::UnaryOperation(arg0) => f.debug_tuple("UnaryOperation").field(arg0).finish(),
            Self::BinaryOperation(arg0) => f.debug_tuple("BinaryOperation").field(arg0).finish(),
            Self::UnaryOperationWithHeld(arg0) => {
                f.debug_tuple("UnaryOperationWithHeld").field(arg0).finish()
            }
            Self::BinaryOperationWithHeld(arg0) => f
                .debug_tuple("BinaryOperationWithHeld")
                .field(arg0)
                .finish(),
            Self::Invoke {
                arg_count,
                stack_size,
                instruction,
            } => f
                .debug_tuple("Invoke")
                .field(arg_count)
                .field(stack_size)
                .field(instruction)
                .finish(),
            Self::InvokeDynamic { arg_count } => {
                f.debug_tuple("InvokeDynamic").field(arg_count).finish()
            }
            Self::InvokeNative(_arg0) => f.debug_tuple("InvokeNative").finish(),
            Self::Return { stack_size } => f.debug_tuple("Return").field(stack_size).finish(),
            Self::ReturnConstant { value, stack_size } => f
                .debug_tuple("ReturnConstant")
                .field(value)
                .field(stack_size)
                .finish(),
            Self::CaptureValues => write!(f, "CaptureValues"),
        }
    }
}
