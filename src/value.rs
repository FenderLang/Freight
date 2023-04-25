use crate::{function::FunctionRef, TypeSystem};
use std::fmt::Debug;

pub trait Value: Clone + Default + Debug + From<FunctionRef<Self::TS>> + PartialEq {
    type TS: TypeSystem<Value = Self>;

    /// Create a new uninitialized (null) reference, will be used for all new stack values
    fn uninitialized_reference() -> Self;

    /// Get the type ID of this value
    fn get_type(&self) -> &<Self::TS as TypeSystem>::TypeId;

    /// Create a deep copy of this value
    fn deep_clone(&self) -> Self;

    /// Create a new reference to this value
    fn dupe_ref(&self) -> Self;

    /// Convert this value into a reference, if it isn't already
    fn into_ref(self) -> Self;

    /// Attempt to cast this value to a function so it can be dynamically invoked
    fn cast_to_function(&self) -> Option<&FunctionRef<Self::TS>>;

    /// Assign to this value
    fn assign(&mut self, value: <Self::TS as TypeSystem>::Value);

    #[cfg(feature = "variadic_functions")]
    /// Create a `Value` type list out of `Vec` of `Value`
    fn gen_list(values: impl IntoIterator<Item = Self>) -> Self;
}
