use std::{
    cell::RefCell,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub type PooledVec<T> = Pooled<T, Vec<T>>;
pub type VecPool<T> = CollectionPool<T, Vec<T>>;
pub type PooledRcSlice<T> = Pooled<T, Rc<[T]>>;
pub type RcSlicePool<T> = CollectionPool<T, Rc<[T]>>;

pub struct CollectionPool<T, C: Poolable<T>> {
    pool: Vec<Vec<C>>,
    elem_type: PhantomData<T>,
    max_cache_per: usize,
}

pub struct Pooled<T, C: Poolable<T>> {
    pool: Rc<RefCell<CollectionPool<T, C>>>,
    collection: C,
}

impl<T, C: Poolable<T> + Clone> Clone for Pooled<T, C> {
    fn clone(&self) -> Self {
        Pooled {
            pool: self.pool.clone(),
            collection: self.collection.clone(),
        }
    }
}

impl<T, C: Poolable<T>> Deref for Pooled<T, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.collection
    }
}

impl<T, C: Poolable<T>> DerefMut for Pooled<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.collection
    }
}

impl<T, C: Poolable<T>> Drop for Pooled<T, C> {
    fn drop(&mut self) {
        self.collection.into_pool(&mut *self.pool.borrow_mut());
    }
}

pub trait Poolable<T>: Sized {
    fn into_pool(&mut self, pool: &mut CollectionPool<T, Self>);
    fn with_capacity(capacity: usize) -> Self;
    fn populate(&mut self, next: impl FnMut() -> T);
    fn capacity(&self) -> usize;
}

impl<T> Poolable<T> for Vec<T> {
    fn into_pool(&mut self, pool: &mut CollectionPool<T, Vec<T>>) {
        pool.insert(std::mem::take(self));
    }

    fn populate(&mut self, mut next: impl FnMut() -> T) {
        self.clear();
        for _ in 0..self.capacity() {
            self.push(next());
        }
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

impl<T: Default> Poolable<T> for Rc<[T]> {
    fn into_pool(&mut self, pool: &mut CollectionPool<T, Self>) {
        if Rc::strong_count(self) + Rc::weak_count(self) != 1 {
            return;
        }
        let clone = self.clone();
        pool.insert(clone);
    }

    fn with_capacity(capacity: usize) -> Self {
        let mut empty_init = vec![];
        for _ in 0..capacity {
            empty_init.push(Default::default());
        }
        empty_init.into()
    }

    fn populate(&mut self, mut next: impl FnMut() -> T) {
        let slice = Rc::get_mut(self).expect("Non-unique rc slice in pool");
        for i in 0..slice.len() {
            slice[i] = next();
        }
    }

    fn capacity(&self) -> usize {
        self.len()
    }
}

pub trait IntoExactSizeIterator: IntoIterator {
    type ExactSizeIter: ExactSizeIterator<Item = Self::Item>;

    fn into_exact_size_iter(self) -> Self::ExactSizeIter;
}

impl<I: IntoIterator> IntoExactSizeIterator for I
where
    I::IntoIter: ExactSizeIterator<Item = I::Item>,
{
    type ExactSizeIter = Self::IntoIter;

    fn into_exact_size_iter(self) -> Self::ExactSizeIter {
        self.into_iter()
    }
}

impl<T, C: Poolable<T> + Debug> Debug for Pooled<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.collection.fmt(f)
    }
}

impl<T, C: Poolable<T>> Default for CollectionPool<T, C> {
    fn default() -> Self {
        Self::with_max_cache_per(1000)
    }
}

impl<T, C: Poolable<T>> CollectionPool<T, C> {
    pub fn with_max_cache_per(max_cache_per: usize) -> Self {
        CollectionPool {
            pool: std::array::from_fn::<_, 100, _>(|_| Vec::with_capacity(max_cache_per)).into(),
            elem_type: PhantomData,
            max_cache_per,
        }
    }

    pub fn insert(&mut self, container: C) {
        self.pool.get_mut(container.capacity()).map(|v| {
            if v.len() < self.max_cache_per {
                v.push(container);
            }
        });
    }

    pub fn request(cell: Rc<RefCell<Self>>, capacity: usize) -> Pooled<T, C> {
        let mut this = cell.borrow_mut();
        let collection = this
            .pool
            .get_mut(capacity)
            .and_then(|cache| cache.pop())
            .unwrap_or_else(|| C::with_capacity(capacity));
        Pooled {
            pool: cell.clone(),
            collection,
        }
    }

    pub fn from_pool(
        cell: Rc<RefCell<Self>>,
        elems: impl IntoExactSizeIterator<Item = T>,
    ) -> Pooled<T, C> {
        let mut iter = elems.into_exact_size_iter();
        let capacity = iter.len();
        let mut container = Self::request(cell, capacity);
        container.collection.populate(|| iter.next().unwrap());
        container
    }

    pub fn from_pool_with_fn(
        cell: Rc<RefCell<Self>>,
        capacity: usize,
        f: impl FnMut() -> T,
    ) -> Pooled<T, C> {
        let mut container = Self::request(cell, capacity);
        container.populate(f);
        container
    }
}
