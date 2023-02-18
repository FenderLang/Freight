/// `Location` referencing temporary held value, in stack, used by the `Expression`.
pub const HELD_VALUE: Location = Location::Addr(HELD_VALUE_ADDRESS);
/// Position in stack of the temporary held value used by the `Expression`.
pub(super) const HELD_VALUE_ADDRESS: usize = 0;
/// `Location` of the `Return` register
pub const RETURN_REGISTER: Location = Location::Register(RegisterId::Return);
/// `Location` of the `RightOperand` register
pub const RIGHT_OPERAND_REGISTER: Location = Location::Register(RegisterId::Return);
#[cfg(feature = "popped_register")]
/// `Location` of the `Popped` register
pub const POPPED_REGISTER: Location = Location::Register(RegisterId::Return);



#[derive(Debug, Clone)]
pub enum Location {
    Register(RegisterId),
    Addr(usize),
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum RegisterId {
    Return,
    RightOperand,
    #[cfg(feature = "popped_register")]
    Popped,
}

impl RegisterId {
    pub(crate) fn id(&self) -> usize {
        match self {
            RegisterId::Return => 0,
            RegisterId::RightOperand => 1,
            #[cfg(feature = "popped_register")]
            RegisterId::Popped => 2,
        }
    }
}
