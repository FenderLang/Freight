use std::ops::{Bound, RangeBounds};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ArgCount {
    Fixed(usize),
    Range {
        min: usize,
        max: usize,
    },
    #[cfg(feature = "variadic_functions")]
    Variadic {
        min: usize,
        max: usize,
    },
}

impl ArgCount {
    #[cfg(not(feature = "variadic_functions"))]
    pub fn new<RB: RangeBounds<usize>>(args: RB) -> ArgCount {
        let min = match args.start_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m + 1),
            Bound::Unbounded => None,
        }
        .unwrap_or(0);

        let max = match args.end_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m - 1),
            Bound::Unbounded => None,
        }
        .unwrap_or(0);
        if min == max {
            ArgCount::Fixed(min)
        } else {
            ArgCount::Range { min, max }
        }
    }
    #[cfg(feature = "variadic_functions")]
    pub fn new<RB: RangeBounds<usize>>(args: RB) -> ArgCount {
        let min = match args.start_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m + 1),
            Bound::Unbounded => None,
        }
        .unwrap_or(0);

        let max = match args.end_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m - 1),
            Bound::Unbounded => None,
        };
        match max {
            Some(max) => ArgCount::Range { min, max },
            None => ArgCount::Variadic { min, max: min },
        }
    }

    #[cfg(feature = "variadic_functions")]
    pub fn new_variadic<RB: RangeBounds<usize>>(args: RB) -> ArgCount {
        let min = match args.start_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m + 1),
            Bound::Unbounded => None,
        }
        .unwrap_or(0);

        let max = match args.end_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m - 1),
            Bound::Unbounded => None,
        };
        match max {
            Some(max) => ArgCount::Variadic { min, max },
            None => ArgCount::Variadic { min, max: min },
        }
    }

    pub fn min(&self) -> usize {
        match self {
            ArgCount::Range { min, max: _ } => *min,
            ArgCount::Fixed(f) => *f,
            #[cfg(feature = "variadic_functions")]
            ArgCount::Variadic { min, max: _ } => *min,
        }
    }

    pub fn max(&self) -> Option<usize> {
        match self {
            ArgCount::Range { min: _, max } => Some(*max),
            ArgCount::Fixed(f) => Some(*f),
            #[cfg(feature = "variadic_functions")]
            ArgCount::Variadic { min: _, max: _ } => None,
        }
    }

    pub fn max_capped(&self) -> usize{
        match self {
            ArgCount::Range { min: _, max } => *max,
            ArgCount::Fixed(f) => *f,
            #[cfg(feature = "variadic_functions")]
            ArgCount::Variadic { min: _, max } => *max,
        }
    }

    pub fn valid_arg_count(&self, val: usize) -> bool {
        match self {
            ArgCount::Range { min, max } => val >= *min && val <= *max,
            ArgCount::Fixed(f) => val == *f,
            #[cfg(feature = "variadic_functions")]
            ArgCount::Variadic { min, max: _ } => val >= *min,
        }
    }

    pub fn stack_size(&self) -> usize {
        match self {
            ArgCount::Fixed(f) => *f,
            ArgCount::Range { min: _, max } => *max,
            #[cfg(feature = "variadic_functions")]
            ArgCount::Variadic { min: _, max } => max + 1,
        }
    }
}
