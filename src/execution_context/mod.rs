use crate::{
    error::FreightError,
    expression::Expression,
    function::{FunctionRef, FunctionType},
    instruction::Instruction,
    operators::{binary::BinaryOperator, unary::UnaryOperator},
    value::Value,
    TypeSystem,
};
use std::{fmt::Debug, rc::Rc};

mod location_identifiers;
pub use location_identifiers::*;

#[derive(Debug)]
pub struct ExecutionContext<TS: TypeSystem> {
    pub(crate) stack: Vec<TS::Value>,
    pub(crate) instruction: usize,
    pub(crate) instructions: Rc<[Instruction<TS>]>,
    pub(crate) frames: Vec<usize>,
    pub(crate) call_stack: Vec<usize>,
    pub(crate) frame: usize,
    pub(crate) registers: [TS::Value; 3],
    pub(crate) entry_point: usize,
    pub(crate) initial_stack_size: usize,
}

impl<TS: TypeSystem> ExecutionContext<TS> {
    pub fn new(
        instructions: Vec<Instruction<TS>>,
        stack_size: usize,
        entry_point: usize,
    ) -> ExecutionContext<TS> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            instruction: 0,
            instructions: instructions.into(),
            frames: vec![],
            call_stack: vec![],
            frame: 0,
            registers: std::array::from_fn(|_| Value::uninitialized_reference()),
            entry_point,
            initial_stack_size: stack_size,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub fn get_register(&self, register: RegisterId) -> &TS::Value {
        &self.registers[register.id()]
    }

    pub fn get_register_mut(&mut self, register: RegisterId) -> &mut TS::Value {
        &mut self.registers[register.id()]
    }

    pub fn get_stack(&self, offset: usize) -> &TS::Value {
        &self.stack[self.frame + offset]
    }

    pub fn get_stack_mut(&mut self, offset: usize) -> &mut TS::Value {
        &mut self.stack[self.frame + offset]
    }

    pub fn get(&self, location: &Location) -> &TS::Value {
        match location {
            Location::Register(reg) => self.get_register(*reg),
            Location::Stack(offset) => self.get_stack(*offset),
            Location::Const(address) => &self.stack[*address],
        }
    }

    pub fn get_mut(&mut self, location: &Location) -> &mut TS::Value {
        match location {
            Location::Register(reg) => self.get_register_mut(*reg),
            Location::Stack(offset) => self.get_stack_mut(*offset),
            Location::Const(address) => &mut self.stack[*address],
        }
    }

    pub fn set(&mut self, offset: usize, value: TS::Value) {
        self.stack[self.frame + offset] = value;
    }

    pub(crate) fn do_return(&mut self, stack_size: usize) {
        if self.frames.is_empty() {
            self.instruction = self.instructions.len();
            return;
        }
        self.frame = self.frames.pop().unwrap();
        self.instruction = self.call_stack.pop().unwrap();
        self.stack.drain((self.stack.len() - stack_size)..);
    }

    pub fn evaluate_expression(
        &mut self,
        expr: &Expression<TS>,
    ) -> Result<TS::Value, FreightError> {
        let result = match expr {
            Expression::RawValue(v) => v.clone(),
            Expression::Variable(addr) => self.get_stack(*addr).clone(),
            Expression::Global(addr) => self.stack[*addr].clone(),
            Expression::BinaryOpEval(op, l, r) => {
                let l = self.evaluate_expression(l)?;
                let r = self.evaluate_expression(r)?;
                op.apply_2(&l, &r)
            }
            Expression::UnaryOpEval(op, v) => {
                let v = self.evaluate_expression(v)?;
                op.apply_1(&v)
            }
            Expression::StaticFunctionCall(func, args) => {
                let mut collected = Vec::with_capacity(func.arg_count);
                for arg in args {
                    collected.push(self.evaluate_expression(arg)?);
                }
                self.call_function(func.clone(), collected)?
            }
            Expression::DynamicFunctionCall(func, args) => {
                let func = self.evaluate_expression(func)?;
                let Some(func) = func.cast_to_function() else {
                    return Err(FreightError::InvalidInvocationTarget);
                };
                let mut collected = Vec::with_capacity(func.arg_count);
                for arg in args {
                    collected.push(self.evaluate_expression(arg)?);
                }
                self.call_function(func.clone(), collected)?
            }
            Expression::FunctionCapture(func) => {
                let FunctionType::CapturingDef(capture) = &func.function_type else {
                    return Err(FreightError::InvalidInvocationTarget);
                };
                let mut func = func.clone();
                func.function_type = FunctionType::CapturingRef(
                    capture
                        .iter()
                        .map(|i| self.get_stack(*i).dupe_ref())
                        .collect::<Vec<_>>()
                        .into(),
                );
                func.into()
            }
        };
        Ok(result)
    }

    pub(crate) fn do_invoke(&mut self, arg_count: usize, stack_size: usize, instruction: usize) {
        self.call_stack.push(self.instruction);
        self.frames.push(self.frame);
        self.instruction = instruction;
        for _ in 0..stack_size - arg_count {
            self.stack.push(Value::uninitialized_reference());
        }
        self.frame = self.stack.len() - stack_size;
    }

    pub fn print_state(&self) {
        println!("registers: {:?}", self.registers);
        println!("stack: {} frame: {}", self.stack.len(), self.frame);
        println!("frame contents: {:?}", &self.stack[self.frame..]);
        println!("stack context: {}", self.frames.len());
        println!("instruction: {:?}", self.instructions[self.instruction]);
        println!("instruction index: {}", self.instruction);
        println!("-----------");
    }
}

/// execution functionality
impl<TS: TypeSystem> ExecutionContext<TS> {
    pub(crate) fn execute_next(&mut self) -> Result<bool, FreightError> {
        #[cfg(feature = "debug_mode")]
        self.print_state();
        if self.instructions.clone()[self.instruction].execute(self)? {
            self.instruction += 1;
        }
        Ok(self.instruction < self.instructions.len())
    }

    pub fn call_function(
        &mut self,
        func: FunctionRef<TS>,
        args: Vec<TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        *self.get_register_mut(RegisterId::Return) = func.into();
        let frame_num = self.frames.len();
        let arg_count = args.len();
        self.stack.extend(args);
        Instruction::InvokeDynamic { arg_count }.execute(self)?;
        while self.frames.len() > frame_num {
            self.execute_next()?;
        }
        self.instruction -= 1;
        Ok(std::mem::take(self.get_register_mut(RegisterId::Return)))
    }

    pub fn run(&mut self) -> Result<&TS::Value, FreightError> {
        self.instruction = self.entry_point;
        self.stack = vec![Value::uninitialized_reference(); self.initial_stack_size];
        while self.execute_next()? {}
        Ok(self.get_register(RegisterId::Return))
    }
}
