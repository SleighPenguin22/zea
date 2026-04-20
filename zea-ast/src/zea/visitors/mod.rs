use indexmap::IndexSet;

pub mod altering;
use crate::zea::visitors::altering::{
    AcceptsAssignmentSimplifier, AssignmentSimplifier, BlockExpander, NodeLabeler, LabelSentinelIDs,
};
use crate::zea::visitors::annotating::{
    AcceptScopeBuilder, IntroducesFreshIdentifiers, ScopeAnnotations,
};
use crate::zea::{BareNodeLabeler, Module};
use altering::AcceptsBlockExpander;

pub mod annotating;

impl Module {
    pub fn give_ids(mut self, last_used_generator: impl NodeLabeler) -> (Module, impl NodeLabeler) {
        let mut labeler = BareNodeLabeler::continue_from_last_id_of(last_used_generator);
        self.label_sentinel_id(&mut labeler);
        (self, labeler)
    }
    pub fn expand_blocks(
        mut self,
        last_used_generator: impl NodeLabeler,
    ) -> (Module, impl NodeLabeler) {
        let mut block_expander = BlockExpander::continue_from_last_id_of(last_used_generator);
        while self.accept_block_expander(&mut block_expander) {
            eprintln!("expanding blocks still...")
        }
        (self, block_expander)
    }

    pub fn simplify_assignments(
        mut self,
        last_used_generator: impl NodeLabeler,
    ) -> (Module, impl NodeLabeler) {
        let mut assignment_simplifier =
            AssignmentSimplifier::continue_from_last_id_of(last_used_generator);
        while self.accept_assignment_simplifier(&mut assignment_simplifier) {
            eprintln!("simplifying assignments still...")
        }
        (self, assignment_simplifier)
    }
    pub fn get_globally_scoped_identifiers(&self) -> IndexSet<String> {
        let mut global_idents: IndexSet<String> = self
            .global_vars
            .iter()
            .flat_map(IntroducesFreshIdentifiers::get_introduced_identifiers)
            .collect();
        let func_idents: IndexSet<String> = self
            .functions
            .iter()
            .flat_map(IntroducesFreshIdentifiers::get_introduced_identifiers)
            .collect();
        let import_idents: IndexSet<String> = self.imports.iter().cloned().collect();
        global_idents.extend(func_idents);
        global_idents.extend(import_idents);
        global_idents
    }
    pub fn annotate_scopes(&self) -> ScopeAnnotations {
        let mut scope_builder = ScopeAnnotations::new();
        self.build_scope_with_parent(69420, &mut scope_builder);
        scope_builder
    }
}
