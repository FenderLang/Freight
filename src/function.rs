use crate::expression::VariableType;
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
    CapturingDef(Vec<VariableType>),
    /// A reference to a function which captures values bundled with those captured values
    CapturingRef(Rc<[TS::Value]>),
}

/// Represents a reference to a function that has been included in a VM
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef<TS: TypeSystem> {
    pub(crate) arg_count: usize,
    pub(crate) stack_size: usize,
    pub(crate) location: usize,
    pub function_type: FunctionType<TS>,
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
    pub fn new_capturing(args: usize, capture: Vec<VariableType>) -> FunctionWriter<TS> {
        Self {
            args,
            stack_size: args,
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
        let var = self.stack_size;
        self.stack_size += 1;
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
            stack_size: self.stack_size,
            arg_count: self.args,
            return_target
        }
    }
}

impl<TS: TypeSystem> FunctionRef<TS> {
    /// The number of arguments the function takes
    pub fn arg_count(&self) -> usize {
        self.arg_count
    }

    /// The total stack space allocated to the function
    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    /// The address of the function in the function table
    pub fn address(&self) -> usize {
        self.location
    }
}
