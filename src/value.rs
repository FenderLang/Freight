use crate::{function::FunctionRef, TypeSystem};
use std::fmt::Debug;

pub trait Value: Clone + Default + Debug {
    type TS: TypeSystem;
    fn get_type(&self) -> &<Self::TS as TypeSystem>::TypeId;
    fn deep_clone(&self) -> Self;
    fn dupe_ref(&self) -> Self;
    fn cast_to_function(&self) -> Option<&FunctionRef>;
}
