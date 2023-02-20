use crate::{function::FunctionRef, TypeSystem};
use std::fmt::Debug;

pub trait Value: Clone + Default + Debug + From<FunctionRef<Self::TS>> + PartialEq {
    type TS: TypeSystem<Value = Self>;
    fn uninitialized_reference() -> Self;
    fn get_type(&self) -> &<Self::TS as TypeSystem>::TypeId;
    fn deep_clone(&self) -> Self;
    fn dupe_ref(&self) -> Self;
    fn cast_to_function(&self) -> Option<&FunctionRef<Self::TS>>;
    fn assign(&mut self, value: <Self::TS as TypeSystem>::Value);
}
