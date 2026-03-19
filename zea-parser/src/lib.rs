#![allow(unused)]

mod grammar;
pub use grammar::ExprParser as ExpressionParser;
pub use grammar::ModParser as ModuleParser;
use zea_ast::zea::{Function, Initialisation};

#[derive(Default, Clone, Copy)]
pub struct NodeIdGenerator {
    cur: usize,
}

pub(crate) enum ModuleItem {
    Init(Initialisation),
    Func(Function),
}

pub(crate) fn separate(items: Vec<ModuleItem>) -> (Vec<Initialisation>, Vec<Function>) {
    let mut globs = vec![];
    let mut funcs = vec![];
    for item in items {
        match item {
            ModuleItem::Init(i) => globs.push(i),
            ModuleItem::Func(f) => funcs.push(f),
        }
    }
    (globs, funcs)
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

#[cfg(test)]
mod tests {
    use crate::{ExpressionParser, ModuleParser, NodeIdGenerator};
    use zea_ast::PrettyAST;

    #[test]
    fn test_expr() {
        let mut generator = NodeIdGenerator::new();
        let p = ExpressionParser::new();
        let s = p.parse(&mut generator, "13 + a * 4 + 2 + 2").unwrap();
        eprintln!("{}", s.pretty_print(0));
    }

    #[test]
    fn test_module() {
        let mut generator = NodeIdGenerator::new();
        let p = ModuleParser::new();
        let s = p
            .parse(
                &mut generator,
                "\
        module main \
        imports {}\
        exports {}\
        fn main() {\
            return 3;\
            return 4;\
        }",
            )
            .unwrap();
        eprintln!("{}", s.pretty_print(0));
    }
}
