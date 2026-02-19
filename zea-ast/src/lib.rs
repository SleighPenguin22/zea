mod cformatting;
pub mod datatype;
mod expression;
mod patterns;
mod statement;
mod toplevel;
mod utils;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::hint;
    use std::hint::assert_unchecked;
    use super::*;
    use datatype::ZeaType;

    #[test]
    fn pointers() {
     hint::select_unpredictable()
    }
}
