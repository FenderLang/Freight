use crate::{
    error::FreightError,
    execution_context::{ExecutionContext, Location, RegisterId, HELD_VALUE_ADDRESS},
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
            } => match location {
                Location::Register(reg) => ctx.registers[reg.id()] = creation_callback(ctx),
                Location::Stack(addr) => *ctx.get_stack_mut(*addr) = creation_callback(ctx),
            },

            SetRaw { location, value } => match location {
                Location::Register(reg) => ctx.registers[reg.id()] = value.clone(),
                Location::Stack(addr) => ctx.set(*addr, value.clone()),
            },

            AssignRaw { location, value } => match location {
                Location::Register(reg) => ctx.registers[reg.id()].assign(value.clone()),
                Location::Stack(addr) => ctx.stack[ctx.frame + addr].assign(value.clone()),
            },

            Assign { from, to } => {
                let new_value = ctx.get(from).clone();
                let dest = match to {
                    Location::Register(reg) => &mut ctx.registers[reg.id()],
                    Location::Stack(addr) => &mut ctx.stack[ctx.frame + addr],
                };
                dest.assign(new_value);
            }

            Move { from, to } => {
                let val = std::mem::take(ctx.get_mut(from));
                *ctx.get_mut(to) = val;
            }
            Swap(location_a, location_b) => {
                let a = std::mem::take(ctx.get_mut(location_a));
                *ctx.get_mut(location_a) = std::mem::take(ctx.get_mut(location_b));
                *ctx.get_mut(location_b) = a;
                //                match (location_a, location_b) {
                //                    (Location::Register(reg1), Location::Register(reg2)) => {
                //                        ctx.registers.swap(reg1.id(), reg2.id())
                //                    }
                //                    (Location::Register(reg), Location::Stack(addr))
                //                    | (Location::Stack(addr), Location::Register(reg)) => std::mem::swap(
                //                            &mut ctx.registers[reg.id()],
                //                        &mut ctx.stack[*addr + ctx.frame],
                //                    ),
                //                    (Location::Stack(addr1), Location::Stack(addr2)) => {
                //                        ctx.stack.swap(*addr1 + ctx.frame, *addr2 + ctx.frame)
                //                    }
                //                }
            }

            PushRaw(value) => ctx.stack.push(value.clone()),
            Push(from) => match from {
                Location::Register(reg) => ctx.stack.push(ctx.registers[reg.id()].clone()),
                Location::Stack(addr) => ctx.stack.push(ctx.get_stack(*addr).clone()),
            },
            #[cfg(feature = "popped_register")]
            Pop => ctx.registers[RegisterId::Popped.id()] = ctx.stack.pop().unwrap_or_default(),
            #[cfg(not(feature = "popped_register"))]
            Pop => ctx.registers[RegisterId::Return.id()] = ctx.stack.pop().unwrap_or_default(),

            UnaryOperation(unary_op) => {
                ctx.registers[RegisterId::Return.id()] =
                    unary_op.apply_1(&ctx.registers[RegisterId::Return.id()]);
            }
            BinaryOperation(binary_op) => {
                ctx.registers[RegisterId::Return.id()] = binary_op.apply_2(
                    &ctx.registers[RegisterId::Return.id()],
                    &ctx.registers[RegisterId::RightOperand.id()],
                );
            }
            UnaryOperationWithHeld(unary_op) => {
                ctx.registers[RegisterId::Return.id()] =
                    unary_op.apply_1(ctx.get_stack(HELD_VALUE_ADDRESS))
            }
            BinaryOperationWithHeld(binary_op) => {
                ctx.registers[RegisterId::Return.id()] = binary_op.apply_2(
                    ctx.get_stack(HELD_VALUE_ADDRESS),
                    &ctx.registers[RegisterId::RightOperand.id()],
                )
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
