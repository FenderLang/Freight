use crate::{
    error::FreightError,
    execution_context::{ExecutionContext, Location, RegisterId},
    function::{FunctionRef, FunctionType, InvokeNative},
    operators::{binary::BinaryOperator, unary::UnaryOperator},
    value::Value,
    TypeSystem,
};
use std::{fmt::Debug, rc::Rc};

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

    /// Uses the unary operation given on the value in the specified location
    UnaryOperation {
        operator: TS::UnaryOp,
        operand: Location,
    },

    /// Uses the given binary operation on the values in the two specified locations
    BinaryOperation {
        operator: TS::BinaryOp,
        left: Location,
        right: Location,
    },

    //TODO: @Redempt add comments for the remaining instructions
    Invoke {
        arg_count: usize,
        stack_size: usize,
        instruction: usize,
    },
    InvokeDynamic {
        arg_count: usize,
    },

    InvokeNative {
        function: Rc<dyn InvokeNative<TS>>,
        arg_count: usize,
    },
    Return {
        stack_size: usize,
    },
    ReturnConstant {
        value: TS::Value,
        stack_size: usize,
    },
    CaptureValues,
}

impl<TS: TypeSystem> Instruction<TS> {
    pub fn execute(&self, ctx: &mut ExecutionContext<TS>) -> Result<bool, FreightError> {
        use Instruction::*;
        let mut increment_index = true;
        match self {
            Create {
                location,
                creation_callback,
            } => *ctx.get_mut(location) = creation_callback(ctx),
            SetRaw { location, value } => *ctx.get_mut(location) = value.clone(),
            AssignRaw { location, value } => ctx.get_mut(location).assign(value.clone()),

            Assign { from, to } => {
                let new_value = ctx.get(from).clone();
                ctx.get_mut(to).assign(new_value);
            }

            Move { from, to } => {
                let val = std::mem::take(ctx.get_mut(from));
                *ctx.get_mut(to) = val;
            }
            Swap(location_a, location_b) => {
                let a = std::mem::take(ctx.get_mut(location_a));
                *ctx.get_mut(location_a) = std::mem::take(ctx.get_mut(location_b));
                *ctx.get_mut(location_b) = a;
            }

            PushRaw(value) => ctx.stack.push(value.clone()),
            Push(from) => ctx.stack.push(ctx.get(from).clone()),
            #[cfg(feature = "popped_register")]
            Pop => *ctx.get_register_mut(RegisterId::Popped) = ctx.stack.pop().unwrap_or_default(),
            #[cfg(not(feature = "popped_register"))]
            Pop => *ctx.get_register_mut(RegisterId::Return) = ctx.stack.pop().unwrap_or_default(),

            UnaryOperation { operator, operand } => {
                *ctx.get_register_mut(RegisterId::Return) = operator.apply_1(ctx.get(operand));
            }

            BinaryOperation {
                operator,
                left,
                right,
            } => {
                *ctx.get_register_mut(RegisterId::Return) =
                    operator.apply_2(ctx.get(left), ctx.get(right))
            }

            Invoke {
                arg_count,
                stack_size,
                instruction,
            } => {
                ctx.do_invoke(*arg_count, *stack_size, *instruction);
                increment_index = false;
            }
            InvokeDynamic { arg_count } => {
                let func = (&ctx.registers[0])
                    .cast_to_function()
                    .ok_or(FreightError::InvalidInvocationTarget)?;
                if *arg_count != func.arg_count {
                    return Err(FreightError::IncorrectArgumentCount {
                        expected: func.arg_count,
                        actual: *arg_count,
                    });
                }
                match &func.function_type {
                    FunctionType::Static => (),
                    FunctionType::CapturingDef(_) => {
                        return Err(FreightError::InvalidInvocationTarget)
                    }
                    FunctionType::CapturingRef(values) => {
                        ctx.stack.extend(values.iter().map(|v| v.dupe_ref()))
                    }
                }
                ctx.do_invoke(func.arg_count, func.stack_size, func.location);
                increment_index = false;
            }
            InvokeNative {
                function,
                arg_count,
            } => {
                let args = ctx.stack.drain(ctx.stack.len() - arg_count..).collect();
                ctx.registers[RegisterId::Return.id()] = function.clone().invoke(ctx, args)?;
            }
            Return { stack_size } => ctx.do_return(*stack_size),
            ReturnConstant { value, stack_size } => {
                ctx.registers[RegisterId::Return.id()] = value.clone();
                ctx.do_return(*stack_size);
            }
            CaptureValues => {
                let func = ctx
                    .get_register(RegisterId::Return)
                    .cast_to_function()
                    .ok_or(FreightError::InvalidInvocationTarget)?;
                let FunctionType::CapturingDef(capture) = &func.function_type else {
                    return Err(FreightError::InvalidInvocationTarget);
                };
                *ctx.get_register_mut(RegisterId::Return) = FunctionRef {
                    function_type: FunctionType::<TS>::CapturingRef(Rc::new(
                        capture
                            .iter()
                            .map(|i| ctx.get_stack(*i).dupe_ref())
                            .collect(),
                    )),
                    ..func.clone()
                }
                .into();
            }
        }
        Ok(increment_index)
    }
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
            Self::UnaryOperation { operator, operand } => f
                .debug_struct("UnaryOperation")
                .field("operator", operator)
                .field("operand", operand)
                .finish(),
            Self::BinaryOperation {
                operator,
                left,
                right,
            } => f
                .debug_struct("BinaryOperation")
                .field("operator", operator)
                .field("left", left)
                .field("right", right)
                .finish(),
            Self::Invoke {
                arg_count,
                stack_size,
                instruction,
            } => f
                .debug_struct("Invoke")
                .field("arg_count", arg_count)
                .field("stack_size", stack_size)
                .field("instruction", instruction)
                .finish(),
            Self::InvokeDynamic { arg_count } => {
                f.debug_tuple("InvokeDynamic").field(arg_count).finish()
            }
            Self::InvokeNative {
                function: _,
                arg_count,
            } => f
                .debug_struct("InvokeNative")
                .field("arg_count", arg_count)
                .finish(),
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
