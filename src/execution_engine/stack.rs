pub struct StackPool<T: Default> {
    stack: Vec<T>,
    base: usize,
}

impl<T: Default> StackPool<T> {
    pub fn with_capacity(capacity: usize) -> StackPool<T> {
        let mut stack = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            stack.push(Default::default());
        }
        StackPool { stack, base: 0 }
    }

    pub fn request<'a>(&mut self, capacity: usize) -> &'a mut [T] {
        if self.base + capacity >= self.stack.len() {
            panic!("Stack overflow {} / {}", self.base, self.stack.len());
        }

        unsafe {
            let ptr = self.stack.as_mut_ptr().offset(self.base as isize);

            self.base += capacity;
            let slice = std::slice::from_raw_parts_mut(ptr, capacity);
            slice
        }
    }

    pub fn release(&mut self, capacity: usize) {
        self.base -= capacity;
    }
}

impl<T: Default> Default for StackPool<T> {
    fn default() -> Self {
        Self::with_capacity(10000)
    }
}
