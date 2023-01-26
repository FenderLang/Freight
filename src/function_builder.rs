struct FunctionBuilder {
    args: usize,
}

impl FunctionBuilder {
    fn new() -> FunctionBuilder {
        Self {
            args: 0,
        }
    }

    fn create_variable(&mut self) -> usize {
        let args = self.args;
        self.args += 1;
        args
    }
}