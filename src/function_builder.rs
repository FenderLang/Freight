#[derive(Debug)]
pub struct FunctionBuilder {
    args: usize,
}

impl FunctionBuilder {
    pub fn new() -> FunctionBuilder {
        Self { args: 0 }
    }

    pub fn create_variable(&mut self) -> usize {
        let args = self.args;
        self.args += 1;
        args
    }
}
