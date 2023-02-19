/// `Location` of the `Return` register
pub const RETURN_REGISTER: Location = Location::Register(RegisterId::Return);
/// `Location` of the `RightOperand` register
pub const RIGHT_OPERAND_REGISTER: Location = Location::Register(RegisterId::RightOperand);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    Register(RegisterId),
    Stack(usize),
    Const(usize),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RegisterId {
    Return,
    RightOperand,
}

impl RegisterId {
    pub(crate) fn id(&self) -> usize {
        match self {
            RegisterId::Return => 0,
            RegisterId::RightOperand => 1,
        }
    }
}
