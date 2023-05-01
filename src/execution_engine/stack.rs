use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub struct StackPool<T: Default> {
    stack: Vec<T>,
    base: usize,
}

pub struct StackSlice<'a, T: Default> {
    slice: &'a mut [T],
    stack: Rc<UnsafeCell<StackPool<T>>>,
}

impl<'a, T: Default> Deref for StackSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

impl<'a, T: Default> DerefMut for StackSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice
    }
}

impl<'a, T: Default> Drop for StackSlice<'a, T> {
    fn drop(&mut self) {
        let mut pool = unsafe { &mut *self.stack.get() };
        pool.base -= self.slice.len();
    }
}

impl<T: Default> StackPool<T> {
    pub fn with_capacity(capacity: usize) -> StackPool<T> {
        let mut stack = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            stack.push(Default::default());
        }
        StackPool { stack, base: 0 }
    }

    pub fn request<'a>(cell: Rc<UnsafeCell<Self>>, capacity: usize) -> StackSlice<'a, T> {
        let this = unsafe { &mut *cell.get() };
        if this.base + capacity >= this.stack.len() {
            panic!("Stack overflow {} / {}", this.base, this.stack.len());
        }

        unsafe {
            let ptr = this.stack.as_mut_ptr().add(this.base);

            this.base += capacity;
            let slice = std::slice::from_raw_parts_mut(ptr, capacity);
            StackSlice { slice, stack: cell }
        }
    }

    pub fn release(this: &UnsafeCell<Self>, capacity: usize) {
        let mut this = unsafe { &mut *this.get() };
        this.base -= capacity;
    }
}

impl<T: Default> Default for StackPool<T> {
    fn default() -> Self {
        Self::with_capacity(10000)
    }
}
