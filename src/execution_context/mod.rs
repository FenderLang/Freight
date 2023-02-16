use crate::{
    error::FreightError,
    function::{FunctionRef, FunctionType},
    instruction::{Instruction, InstructionWrapper},
    value::Value,
    BinaryOperator, TypeSystem, UnaryOperator,
};
use std::fmt::Debug;

mod location_identifiers;
pub use location_identifiers::*;

/// `LocationType` referencing temporary held value, in stack, used by the `Expression`.
pub const HELD_VALUE: Location = Location::Addr(0);
/// Position in stack of the temporary held value used by the `Expression`.
const HELD_VALUE_ADDRESS: usize = 0;
/// `LocationType` of the `Return` register
pub const RETURN_REGISTER: Location = Location::Register(RegisterId::Return);
/// `LocationType` of the `RightOperand` register
pub const RIGHT_OPERAND_REGISTER: Location = Location::Register(RegisterId::Return);
#[cfg(feature = "popped_register")]
/// `LocationType` of the `Popped` register
pub const POPPED_REGISTER: Location = Location::Register(RegisterId::Return);

#[derive(Debug)]
pub struct ExecutionContext<TS: TypeSystem> {
    stack: Vec<TS::Value>,
    instructions: Vec<Instruction<TS>>,
    instruction: usize,
    frames: Vec<usize>,
    call_stack: Vec<usize>,
    frame: usize,
    registers: [TS::Value; 3],
    entry_point: usize,
    initial_stack_size: usize,
}

impl<TS: TypeSystem> ExecutionContext<TS> {
    pub fn new(
        instructions: Vec<Instruction<TS>>,
        stack_size: usize,
        entry_point: usize,
    ) -> ExecutionContext<TS> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            initial_stack_size: stack_size,
            instructions,
            instruction: 0,
            frames: vec![],
            call_stack: vec![],
            frame: 0,
            registers: std::array::from_fn(|_| Value::uninitialized_reference()),
            entry_point,
        }
    }

    pub fn get_register(&self, register: RegisterId) -> &TS::Value {
        &self.registers[register.id()]
    }

    pub fn get_register_mut(&mut self, register: RegisterId) -> &mut TS::Value {
        &mut self.registers[register.id()]
    }

    pub fn get(&self, offset: usize) -> &TS::Value {
        &self.stack[self.frame + offset]
    }

    pub fn set(&mut self, offset: usize, value: TS::Value) {
        self.stack[self.frame + offset] = value;
    }

    pub fn get_mut(&mut self, offset: usize) -> &mut TS::Value {
        &mut self.stack[self.frame + offset]
    }

    fn do_return(&mut self, stack_size: usize) {
        self.frame = self.frames.pop().unwrap();
        self.instruction = self.call_stack.pop().unwrap();
        self.stack.drain((self.stack.len() - stack_size)..);
    }

    fn do_invoke(&mut self, arg_count: usize, stack_size: usize, instruction: usize) {
        self.call_stack.push(self.instruction);
        self.frames.push(self.frame);
        self.instruction = instruction;
        // Subtract 1 to account for the held value slot
        for _ in 0..stack_size - arg_count - 1 {
            self.stack.push(Value::uninitialized_reference());
        }
        self.frame = self.stack.len() - stack_size;
    }
}

/// execution functionality
impl<TS: TypeSystem> ExecutionContext<TS> {
    pub fn run(&mut self) -> Result<(), FreightError> {
        self.instruction = self.entry_point;
        self.stack = vec![];
        for _ in 0..self.initial_stack_size {
            self.stack.push(Value::uninitialized_reference());
        }
        while self.instruction < self.instructions.len() {
            if self.execute(InstructionWrapper::InstructionLocation(self.instruction))? {
                self.instruction += 1;
            }
        }
        Ok(())
    }

    pub fn execute(&mut self, ins: InstructionWrapper<TS>) -> Result<bool, FreightError> {
        use Instruction::*;
        let (instruction, mut increment_index) = match &ins {
            InstructionWrapper::RawInstruction(i) => (i, false),
            InstructionWrapper::InstructionLocation(index) => (&self.instructions[*index], true),
        };

        match instruction {
            Create {
                location,
                creation_callback,
            } => match location {
                Location::Register(reg) => self.registers[reg.id()] = creation_callback(self),
                Location::Addr(addr) => *self.get_mut(*addr) = creation_callback(self),
            },
            SetRaw { location, value } => match location {
                Location::Register(reg) => self.registers[reg.id()] = value.clone(),
                Location::Addr(addr) => self.set(*addr, value.clone()),
            },

            Move { from, to } => match (from, to) {
                (Location::Register(from), Location::Register(to)) => {
                    self.registers[to.id()] = std::mem::take(&mut self.registers[from.id()])
                }
                (Location::Register(from), Location::Addr(to)) => {
                    *self.get_mut(*to) = std::mem::take(&mut self.registers[from.id()])
                }
                (Location::Addr(from), Location::Register(to)) => {
                    self.registers[to.id()] = self.get(*from).clone()
                }
                (Location::Addr(from), Location::Addr(to)) => {
                    *self.get_mut(*to) = self.get(*from).clone()
                }
            },

            PushRaw(value) => self.stack.push(value.clone()),
            Push(from) => match from {
                Location::Register(reg) => self.stack.push(self.registers[reg.id()].clone()),
                Location::Addr(addr) => self.stack.push(self.get(*addr).clone()),
            },
            #[cfg(feature = "popped_register")]
            Pop => self.registers[RegisterId::Popped.id()] = self.stack.pop().unwrap_or_default(),
            #[cfg(not(feature = "popped_register"))]
            Pop => self.registers[RegisterId::Return.id()] = self.stack.pop().unwrap_or_default(),

            UnaryOperation(unary_op) => {
                self.registers[RegisterId::Return.id()] =
                    unary_op.apply_1(&self.registers[RegisterId::Return.id()]);
            }
            BinaryOperation(binary_op) => {
                self.registers[RegisterId::Return.id()] = binary_op.apply_2(
                    &self.registers[RegisterId::Return.id()],
                    &self.registers[RegisterId::RightOperand.id()],
                );
            }
            UnaryOperationWithHeld(unary_op) => {
                self.registers[RegisterId::Return.id()] =
                    unary_op.apply_1(self.get(HELD_VALUE_ADDRESS))
            }
            BinaryOperationWithHeld(binary_op) => {
                self.registers[RegisterId::Return.id()] = binary_op.apply_2(
                    self.get(HELD_VALUE_ADDRESS),
                    &self.registers[RegisterId::RightOperand.id()],
                )
            }

            Invoke(arg_count, stack_size, instruction) => {
                self.do_invoke(*arg_count, *stack_size, *instruction);
                increment_index = false;
            }
            InvokeDynamic(arg_count) => {
                let func = (&self.registers[0])
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
                        self.stack.extend(values.iter().map(|v| v.dupe_ref()))
                    }
                }
                self.do_invoke(func.arg_count, func.stack_size, func.location);
                increment_index = false;
            }
            InvokeNative(func) => self.registers[RegisterId::Return.id()] = func(self),
            Return(stack_size) => self.do_return(*stack_size),
            ReturnConstant(c, stack_size) => {
                self.registers[RegisterId::Return.id()] = c.clone();
                self.do_return(*stack_size);
            }
            CaptureValues => {
                let func = self
                    .get_register(RegisterId::Return)
                    .cast_to_function()
                    .ok_or(FreightError::InvalidInvocationTarget)?;
                let FunctionType::CapturingDef(capture) = &func.function_type else {
                    return Err(FreightError::InvalidInvocationTarget);
                };
                *self.get_register_mut(RegisterId::Return) = FunctionRef {
                    function_type: FunctionType::<TS>::CapturingRef(
                        capture.iter().map(|i| self.get(*i).dupe_ref()).collect(),
                    ),
                    ..func.clone()
                }
                .into();
            }
        }
        Ok(increment_index)
    }
}
