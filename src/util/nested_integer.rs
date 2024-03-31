#[derive(Debug, PartialEq, Eq)]
pub enum NestedInteger {
    Int(i32),
    List(Vec<NestedInteger>),
}

impl NestedInteger {
    #[inline]
    pub fn new_int(value: i32) -> Self {
        NestedInteger::Int(value)
    }

    #[inline]
    pub fn new_list(list: Vec<NestedInteger>) -> Self {
        NestedInteger::List(list)
    }
}

#[macro_export]
macro_rules! nested_int {
    // Match a single integer and wrap it with `NestedInteger::Int`
    ($e:expr) => {
        NestedInteger::Int($e)
    };

    // Match a list of elements (integers or nested lists) and wrap it with `NestedInteger::List`
    ([$($inner:tt),* $(,)?]) => {
        NestedInteger::List(vec![$(nested_int!($inner)),*])
    };
}
