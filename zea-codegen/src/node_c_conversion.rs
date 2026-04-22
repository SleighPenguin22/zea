/// vec![], but it boxes each of its expressions
macro_rules! vecboxed {
    ($($e:expr),*) => {
        vec![$(Box::new($e)),*]
    };
}
