use crate::{execution_engine::Function, expression::Expression, TypeSystem};

use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug)]
pub struct FunctionWriter<TS: TypeSystem> {
    pub(crate) stack_size: usize,
    pub(crate) args: usize,
    pub(crate) expressions: Vec<Expression<TS>>,
    pub(crate) function_type: FunctionType<TS>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FunctionType<TS: TypeSystem> {
    /// Static reference to a function, which can't capture any values.
    Static,
    /// Reference to a function which captures values, but hasn't been initialized with those values.
    CapturingDef(Vec<usize>),
    /// A reference to a function which captures values bundled with those captured values
    CapturingRef(Rc<[TS::Value]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef<TS: TypeSystem> {
    pub(crate) arg_count: usize,
    pub(crate) stack_size: usize,
    pub(crate) location: usize,
    pub(crate) function_type: FunctionType<TS>,
}

impl<TS: TypeSystem> FunctionWriter<TS> {
    pub fn new(args: usize) -> FunctionWriter<TS> {
        Self {
            args,
            stack_size: args,
            expressions: vec![],
            function_type: FunctionType::Static,
        }
    }

    /// Create a capturing function (closure)
    /// args: How many arguments the function will take
    /// capture: What items in the current stack frame to capture when creating an instance
    pub fn new_capturing(args: usize, capture: Vec<usize>) -> FunctionWriter<TS> {
        Self {
            args,
            stack_size: args,
            expressions: vec![],
            function_type: FunctionType::CapturingDef(capture),
        }
    }

    pub fn create_variable(&mut self) -> usize {
        let var = self.stack_size;
        self.stack_size += 1;
        var
    }

    pub fn assign_value(&mut self, var: usize, expr: Expression<TS>) {
        self.expressions
            .push(Expression::AssignStack(var, expr.into()));
    }

    pub fn evaluate_expression(&mut self, expr: Expression<TS>) {
        self.expressions.push(expr);
    }

    pub fn captured_stack_offset(&self, captured: usize) -> usize {
        captured
    }

    pub fn argument_stack_offset(&self, arg: usize) -> usize {
        arg + if let FunctionType::CapturingDef(capture) = &self.function_type {
            capture.len()
        } else {
            0
        }
    }

    pub fn return_expression(&mut self, expr: Expression<TS>) {
        self.evaluate_expression(expr);
    }

    pub fn build(self) -> Function<TS> {
        Function {
            expressions: self.expressions,
            stack_size: self.stack_size,
            arg_count: self.args,
        }
    }
}

impl<TS: TypeSystem> FunctionRef<TS> {
    pub fn arg_count(&self) -> usize {
        self.arg_count
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn address(&self) -> usize {
        self.location
    }
}
