use super::{arg_count::ArgCount, FunctionType};
use crate::{expression::NativeFunction, TypeSystem};

#[derive(Debug, Clone)]
pub struct StackLayout(u128);

impl StackLayout {
    pub fn all_alloc() -> StackLayout {
        StackLayout(u128::MAX)
    }

    pub fn no_alloc() -> StackLayout {
        StackLayout(0)
    }

    pub fn set_alloc(&mut self, slot: usize) {
        self.0 |= 1 << slot;
    }

    pub fn set_stack(&mut self, slot: usize) {
        self.0 &= !(1 << slot);
    }

    pub fn is_alloc(&self, slot: usize) -> bool {
        (self.0 & (1 << slot)) != 0
    }
}

/// Represents a reference to a function that has been included in a VM
#[derive(Debug, Clone)]
pub struct FunctionRef<TS: TypeSystem> {
    pub(crate) arg_count: ArgCount,
    pub(crate) stack_size: usize,
    pub(crate) location: usize,
    pub function_type: FunctionType<TS>,
    pub layout: StackLayout,
}

impl<TS: TypeSystem> PartialEq for FunctionRef<TS> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.function_type, &other.function_type) {
            (FunctionType::Native(_), FunctionType::Native(_)) => self.location == other.location,
            (FunctionType::Native(_), _) | (_, FunctionType::Native(_)) => false,
            _ => self.location == other.location,
        }
    }
}

impl<TS: TypeSystem> FunctionRef<TS> {
    /// Create a new native function
    pub fn new_native(id: usize, func: NativeFunction<TS>, arg_count: ArgCount) -> Self {
        Self {
            arg_count,
            location: id,
            stack_size: arg_count.stack_size(),
            function_type: FunctionType::Native(func),
            layout: StackLayout::no_alloc(),
        }
    }

    /// The number of arguments the function takes
    pub fn arg_count(&self) -> ArgCount {
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
