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
