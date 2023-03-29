use super::{arg_count::ArgCount, FunctionType};
use crate::TypeSystem;

/// Represents a reference to a function that has been included in a VM
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef<TS: TypeSystem> {
    pub(crate) arg_count: ArgCount,
    pub(crate) stack_size: usize,
    pub(crate) location: usize,
    pub(crate) variable_count: usize,
    pub function_type: FunctionType<TS>,
}

impl<TS: TypeSystem> FunctionRef<TS> {
    /// The number of arguments the function takes
    pub fn arg_count(&self) -> usize {
        // self.arg_count
        todo!()
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
