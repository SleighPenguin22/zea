/// This package holds the AST nodes for the Zea language, along with the target C AST.
pub mod c;
pub mod zea;

pub trait PrettyAST {
    fn pretty_print(&self, depth: usize) -> String;
    fn depth_str(depth: usize) -> String {
        " ".repeat(depth * 2)
    }
}
