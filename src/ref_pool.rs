use std::{
    cell::RefCell,
    collections::VecDeque,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub trait PoolableRef: Default + ShouldRecycle + Clone {}
impl<T: PoolableRef + Clone> PoolableRef for T {}

impl<T> ShouldRecycle for Rc<T> {
    fn should_recycle(&self) -> bool {
        Rc::strong_count(self) + Rc::weak_count(self) == 0
    }
}

pub struct PooledRef<T: PoolableRef> {
    val: T,
    pool: Rc<RefCell<RefPool<T>>>,
}

impl<T: PoolableRef> Deref for PooledRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T: PoolableRef> DerefMut for PooledRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T: PoolableRef> Drop for PooledRef<T> {
    fn drop(&mut self) {
        if self.should_recycle() {
            self.pool.borrow_mut().insert(self.clone())
        }
    }
}

pub trait ShouldRecycle {
    fn should_recycle(&self) -> bool;
}

pub struct RefPool<T: PoolableRef> {
    pool: VecDeque<T>,
}

impl<T: PoolableRef> Default for RefPool<T> {
    fn default() -> Self {
        Self::with_max_capacity(1000)
    }
}

impl<T: PoolableRef> RefPool<T> {
    pub fn with_max_capacity(max_capacity: usize) -> RefPool<T> {
        RefPool {
            pool: VecDeque::with_capacity(max_capacity),
        }
    }

    pub fn request(cell: Rc<RefCell<Self>>) -> PooledRef<T> {
        let val = cell.borrow_mut().pool.pop_back().unwrap_or_default();
        PooledRef { val, pool: cell }
    }

    pub fn insert(&mut self, val: T) {
        if self.pool.len() < self.pool.capacity() {
            self.pool.push_back(val);
        }
    }
}
