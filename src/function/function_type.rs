use crate::expression::{NativeFunction, VariableType};
use crate::TypeSystem;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum FunctionType<TS: TypeSystem> {
    /// Static reference to a function, which can't capture any values.
    Static,
    /// Reference to a function which captures values, but hasn't been initialized with those values.
    CapturingDef(Vec<VariableType>),
    /// Reference to a function which captures values bundled with those captured values
    CapturingRef(Rc<[TS::Value]>),
    /// Reference to a native function
    Native(NativeFunction<TS>),
}
