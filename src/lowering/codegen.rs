use crate::lowering::{LoweredConstInitialisation, LoweringResult};

pub trait CNode {
    fn emit_c(&self) -> String;
}

impl CNode for Vec<Box<dyn CNode>> {
    fn emit_c(&self) -> String {
        self.iter().map(|c| c.emit_c()).collect()
    }
}

pub fn canoncalize_zea_identifier(identifier: &str) -> String {
    identifier
        .replace("-", "_")
        .replace("!", "_bang_")
        .replace("?", "_maybe_")
        .replace("__", "_")
        .trim_end_matches("_")
        .to_string()
}

#[cfg(test)]
mod tests {

    #[test]
    fn canonicalize_zea_identifier() {
        use super::canoncalize_zea_identifier as c;
        let s1 = "even?";
        let s2 = "kebab-case";
        let s3 = "map!";
        let s4 = "unify-types?!";
        let s5 = "unify-types?_!";
        assert_eq!(c(s1), "even_maybe");
        assert_eq!(c(s2), "kebab_case");
        assert_eq!(c(s3), "map_bang");
        assert_eq!(c(s4), "unify_types_maybe_bang");
        assert_eq!(c(s4), "unify_types_maybe_bang");
    }
}
