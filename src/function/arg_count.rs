use std::ops::{Bound, RangeBounds};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ArgCount {
    Range {
        min: Option<usize>,
        max: Option<usize>,
        // error_range: Option<>
    },
    Fixed(usize),
}

impl ArgCount {
    pub fn new<RB: RangeBounds<usize>>(args: RB) -> ArgCount {
        let min = match args.start_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m + 1),
            Bound::Unbounded => None,
        };

        let max = match args.end_bound() {
            Bound::Included(m) => Some(*m),
            Bound::Excluded(m) => Some(m - 1),
            Bound::Unbounded => None,
        };
        if let (Some(min), Some(max)) = (min, max) {
            assert!(min <= max);
            if min == max {
                return ArgCount::Fixed(min);
            }
        }
        ArgCount::Range { min, max }
    }

    pub fn min(&self) -> usize {
        match self {
            ArgCount::Range { min, max: _ } => min.unwrap_or(0),
            ArgCount::Fixed(f) => *f,
        }
    }

    pub fn max(&self) -> Option<usize> {
        match self {
            ArgCount::Range { min: _, max } => *max,
            ArgCount::Fixed(f) => Some(*f),
        }
    }

    pub fn contains(&self, val: usize) -> bool {
        match self {
            ArgCount::Range {
                min: Some(min),
                max: Some(max),
            } => val >= *min && val <= *max,
            ArgCount::Range {
                min: None,
                max: Some(max),
            } => val <= *max,
            ArgCount::Range {
                min: Some(min),
                max: None,
            } => val >= *min,
            ArgCount::Range {
                min: None,
                max: None,
            } => true,
            ArgCount::Fixed(f) => val == *f,
        }
    }

    // pub fn trimmed_stack_size(&self) -> usize {
    //     match self {
    //         ArgCount::Fixed(f) => *f,
    //         ArgCount::Range {
    //             min: _,
    //             max: Some(max),
    //         } => *max,
    //         ArgCount::Range {
    //             min: None,
    //             max: None,
    //         } => 0,
    //         ArgCount::Range {
    //             min: Some(min),
    //             max: _,
    //         } => *min,
    //     }
    // }
    pub fn stack_size(&self) -> usize {
        match self {
            ArgCount::Fixed(f) => *f,

            ArgCount::Range {
                min: None,
                max: Some(max),
            } => *max ,
            ArgCount::Range {
                min: None,
                max: None,
            } => 1,
            ArgCount::Range {
                min: Some(min),
                max: _,
            } => *min + 1,
        }
    }
}
