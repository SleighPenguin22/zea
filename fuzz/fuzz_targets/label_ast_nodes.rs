#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use zea_ipr::ast::visitors::IPRVisitor;
use zea_ipr::ast::{
    ZeaNodeQuery,
    ipr::*,
    visitors::{altering::AssignmentExpander, annotating::IPRHasUniqueIDs},
};
fuzz_target!(|data: &[u8]| {
    let mut uniques = IPRHasUniqueIDs::new();
    if let Ok(mut m) = IPRModule::arbitrary(&mut Unstructured::new(data)) {
        let label = m.label_nodes();
        assert!(uniques.visit_module(&m).is_ok());
        // println!("{m:?}");
        uniques.reset();
        let res = m.transform_self_with::<AssignmentExpander>(label);
        assert!(res.is_ok());
        let res = uniques.visit_module(&m);
        // println!("{m:?}");
        if let _e @ Err(id) = res {
            let _query = ZeaNodeQuery::query_ipr_node(id, &m);
        }
        assert!(res.is_ok());
    }
});
