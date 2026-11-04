//! Approved placement budget; original characterization remains separate.
use crate::ir::{CompiledIR, CoreOp};
use crate::work_observations_af_perf_002 as work;

#[test]
fn hierarchy_placement_bounded_work() {
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    let mut excess = Vec::new();
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
            let counts = work::snapshot();
            let requests = 2 * classes * members;
            let lookup_work =
                counts.hierarchy_class_comparisons + counts.hierarchy_placement_map_probes;
            if lookup_work > requests {
                excess.push((classes, members, definitions_first, lookup_work, requests));
            }
            eprintln!(
                "AF002_HIERARCHY classes={classes} members_per_kind={members} definitions_first={definitions_first} instructions={} owner_comparisons={} map_probes={} placement_requests={requests} us={elapsed} boundary=checked-hierarchy",
                ir.instructions.len(),
                counts.hierarchy_class_comparisons,
                counts.hierarchy_placement_map_probes
            );
        }
    }
    assert!(
        excess.is_empty(),
        "Hierarchy placement work exceeds one lookup per member: {excess:?}"
    );
}

#[test]
fn hierarchy_placement_forward_and_refusal_equivalence() {
    use crate::ir::hierarchical::try_ir_to_hierarchical;
    let make = |instructions| CompiledIR {
        file_id: "placement".into(),
        version: 1,
        instructions,
    };
    let ir = make(vec![
        CoreOp::DefClass("B".into(), "SameName".into()),
        CoreOp::DefMethod("A".into(), "M0".into(), "queued".into()),
        CoreOp::DefField("A".into(), "F0".into(), "queued".into()),
        CoreOp::DefClass("A".into(), "SameName".into()),
        CoreOp::DefMethod("A".into(), "M1".into(), "later".into()),
        CoreOp::DefField("A".into(), "F1".into(), "later".into()),
    ]);
    let hierarchy = try_ir_to_hierarchical(&ir).unwrap();
    assert_eq!(
        hierarchy
            .classes
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        ["B", "A"]
    );
    assert!(hierarchy.classes[0].methods.is_empty());
    assert!(hierarchy.classes[0].fields.is_empty());
    assert_eq!(
        hierarchy.classes[1]
            .methods
            .iter()
            .map(|m| (m.id.as_str(), m.name.as_str()))
            .collect::<Vec<_>>(),
        [("M0", "queued"), ("M1", "later")]
    );
    assert_eq!(
        hierarchy.classes[1]
            .fields
            .iter()
            .map(|f| (f.id.as_str(), f.name.as_str()))
            .collect::<Vec<_>>(),
        [("F0", "queued"), ("F1", "later")]
    );
    for duplicate_name in ["First", "Different"] {
        work::reset();
        let error = try_ir_to_hierarchical(&make(vec![
            CoreOp::DefClass("A".into(), "First".into()),
            CoreOp::DefMethod("A".into(), "M".into(), "run".into()),
            CoreOp::DefClass("A".into(), duplicate_name.into()),
        ]))
        .unwrap_err();
        assert!(matches!(
            error,
            crate::ir::hierarchical::HierarchicalProjectionError::DuplicateIdentity { .. }
        ));
        assert_eq!(
            work::snapshot().hierarchy_class_comparisons,
            0,
            "validation rejects before placement"
        );
    }
    for member in [
        CoreOp::DefMethod("Missing".into(), "M".into(), "run".into()),
        CoreOp::DefField("Missing".into(), "F".into(), "field".into()),
    ] {
        let error = try_ir_to_hierarchical(&make(vec![member])).unwrap_err();
        assert!(matches!(
            error,
            crate::ir::hierarchical::HierarchicalProjectionError::UnresolvedIdentity { .. }
        ));
    }
}
