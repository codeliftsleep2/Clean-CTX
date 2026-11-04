//! Characterize real checked hierarchy placement without a hypothetical budget.
use crate::ir::{CompiledIR, CoreOp};
use crate::work_observations_af_perf_002 as work;

#[test]
fn af002_hierarchy_owner_placement_growth() {
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    for (classes, members) in axes {
        for definitions_first in [false, true] {
            let definitions: Vec<_> = (0..classes)
                .map(|c| CoreOp::DefClass(format!("C{c}"), format!("Owner{c}")))
                .collect();
            let mut instructions = Vec::new();
            if definitions_first {
                instructions.extend(definitions.clone());
            }
            for (c, definition) in definitions.into_iter().enumerate() {
                if !definitions_first {
                    instructions.push(definition);
                }
                for m in 0..members {
                    instructions.push(CoreOp::DefMethod(
                        format!("C{c}"),
                        format!("M{c}_{m}"),
                        format!("run{m}"),
                    ));
                    instructions.push(CoreOp::DefField(
                        format!("C{c}"),
                        format!("F{c}_{m}"),
                        format!("field{m}"),
                    ));
                }
            }
            let ir = CompiledIR {
                file_id: "hierarchy-work".into(),
                version: 1,
                instructions,
            };
            work::reset();
            let start = std::time::Instant::now();
            let hierarchy = crate::ir::hierarchical::try_ir_to_hierarchical(&ir).unwrap();
            let elapsed = start.elapsed().as_micros();
            assert_eq!(hierarchy.classes.len(), classes);
            for (c, class) in hierarchy.classes.iter().enumerate() {
                assert_eq!(class.id, format!("C{c}"));
                assert_eq!(class.name, format!("Owner{c}"));
                assert_eq!(class.methods.len(), members);
                assert_eq!(class.fields.len(), members);
                for m in 0..members {
                    assert_eq!(class.methods[m].id, format!("M{c}_{m}"));
                    assert_eq!(class.methods[m].name, format!("run{m}"));
                    assert_eq!(class.fields[m].id, format!("F{c}_{m}"));
                    assert_eq!(class.fields[m].name, format!("field{m}"));
                }
            }
            eprintln!(
                "AF002_HIERARCHY classes={classes} members_per_kind={members} definitions_first={definitions_first} instructions={} owner_comparisons={} us={elapsed} boundary=checked-hierarchy",
                ir.instructions.len(),
                work::snapshot().hierarchy_class_comparisons
            );
        }
    }
}
