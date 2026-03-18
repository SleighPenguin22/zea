#![allow(unused)]
mod grammar;
use grammar::ExprParser;

#[derive(Default, Clone, Copy)]
pub struct NodeIdGenerator {
    cur: usize,
}
impl NodeIdGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&mut self) -> usize {
        let cur = self.cur;
        self.cur += 1;
        cur
    }
}
