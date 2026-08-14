#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use zea_ipr::ast::{ipr::IPRModule, visitors::altering::AssignmentExpander};

fuzz_target!(|data: &[u8]| {
    let mut ud = Unstructured::new(data);
    if let Ok(mut m) = IPRModule::arbitrary(&mut ud) {
        let label = m.label_nodes();
        let _res = m
            .transform_self_with::<AssignmentExpander>(label)
            .expect("assignment expander");
        todo!()
    }
});
