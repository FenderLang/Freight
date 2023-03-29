use crate::expression::VariableType;
use crate::{expression::Expression, TypeSystem};
use std::fmt::Debug;

use super::arg_count::ArgCount;
use super::{Function, FunctionType};

#[derive(Debug)]
pub struct FunctionWriter<TS: TypeSystem> {
    pub(crate) variable_count: usize,
    pub(crate) args: ArgCount,
    pub(crate) expressions: Vec<Expression<TS>>,
    pub(crate) function_type: FunctionType<TS>,
}

impl<TS: TypeSystem> FunctionWriter<TS> {
    pub fn new(args: ArgCount) -> FunctionWriter<TS> {
        Self {
            args,
            variable_count: 0,
            expressions: vec![],
            function_type: FunctionType::Static,
        }
    }

    /// Create a capturing function (closure)
    /// args: How many arguments the function will take
    /// capture: What items in the current stack frame to capture when creating an instance
    pub fn new_capturing(args: ArgCount, capture: Vec<VariableType>) -> FunctionWriter<TS> {
        Self {
            args,
            variable_count: 0,
            expressions: vec![],
            function_type: FunctionType::CapturingDef(capture),
        }
    }

    /// Convert this into a capturing function which will capture the specified values from its environment
    pub fn set_captures(&mut self, capture: Vec<VariableType>) {
        self.function_type = FunctionType::CapturingDef(capture);
    }

    /// Create a new variable in the scope of this function and return its address
    pub fn create_variable(&mut self) -> usize {
        let var = self.args.stack_size() + self.variable_count;
        self.variable_count += 1;
        var
    }

    /// Add an expression to be evaluated when this function is called
    pub fn evaluate_expression(&mut self, expr: Expression<TS>) {
        self.expressions.push(expr);
    }

    /// Create a function from this writer
    pub fn build(self, return_target: usize) -> Function<TS> {
        Function {
            expressions: self.expressions,
            variable_count: self.variable_count,
            arg_count: self.args,
            return_target,
        }
    }
}
