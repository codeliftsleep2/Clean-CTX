// src/tests/ir/patterns.rs
//
// Tests for Phase H: Pattern Compression (Consumptive Recognizer).

use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::{
    PatternOp, CompressingPatternRecognizer, CompressedItem,
};

fn defclass(id: &str, name: &str) -> CoreOp {
    CoreOp::DefClass(id.into(), name.into())
}

fn defmethod(cid: &str, mid: &str, name: &str) -> CoreOp {
    CoreOp::DefMethod(cid.into(), mid.into(), name.into())
}

fn param(mid: &str, pid: &str, ty: &str, name: &str) -> CoreOp {
    CoreOp::Param(mid.into(), pid.into(), ty.into(), name.into())
}

fn ret(mid: &str, ty: &str) -> CoreOp {
    CoreOp::Return(mid.into(), ty.into())
}

fn flags(tid: &str, fs: &[&str]) -> CoreOp {
    CoreOp::Flags(tid.into(), fs.iter().map(|s| s.to_string()).collect())
}

fn injects(cid: &str, deps: &[&str]) -> CoreOp {
    CoreOp::Injects(cid.into(), deps.iter().map(|s| s.to_string()).collect())
}

// ── PatternOp round-trip ──────────────────────────────────────

#[test]
fn pattern_op_constructor_to_from_tuple() {
    let pat = PatternOp::Constructor {
        class_id: "C1".into(),
        method_id: "M1".into(),
        deps: vec!["S1".into(), "S2".into()],
    };
    let t = pat.to_tuple();
    assert_eq!(t[0], "PAT");
    assert_eq!(t[1], "CTOR");
    assert_eq!(t[2], "C1");
    assert_eq!(t[3], "M1");
    assert_eq!(t[4], "S1");
    assert_eq!(t[5], "S2");
    let back = PatternOp::from_tuple(&t).expect("should round-trip");
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_empty_ctor_round_trip() {
    let pat = PatternOp::EmptyConstructor {
        class_id: "C1".into(),
        method_id: "M2".into(),
    };
    let t = pat.to_tuple();
    assert_eq!(t, vec!["PAT", "EMPTY_CTOR", "C1", "M2"]);
    let back = PatternOp::from_tuple(&t).unwrap();
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_observable_round_trip() {
    let pat = PatternOp::Observable {
        class_id: "C1".into(),
        method_id: "M1".into(),
        return_type: "$P".into(),
    };
    let t = pat.to_tuple();
    assert_eq!(t, vec!["PAT", "OBSERVABLE", "C1", "M1", "$P"]);
    let back = PatternOp::from_tuple(&t).unwrap();
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_promise_round_trip() {
    let pat = PatternOp::Promise {
        class_id: "C1".into(),
        method_id: "M1".into(),
        return_type: "$P".into(),
    };
    let t = pat.to_tuple();
    assert_eq!(t, vec!["PAT", "PROMISE", "C1", "M1", "$P"]);
    let back = PatternOp::from_tuple(&t).unwrap();
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_getter_round_trip() {
    let pat = PatternOp::Getter {
        class_id: "C1".into(),
        method_id: "M1".into(),
        property: "fullName".into(),
    };
    let t = pat.to_tuple();
    assert_eq!(t, vec!["PAT", "GETTER", "C1", "M1", "fullName"]);
    let back = PatternOp::from_tuple(&t).unwrap();
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_setter_round_trip() {
    let pat = PatternOp::Setter {
        class_id: "C1".into(),
        method_id: "M1".into(),
        property: "fullName".into(),
    };
    let t = pat.to_tuple();
    assert_eq!(t, vec!["PAT", "SETTER", "C1", "M1", "fullName"]);
    let back = PatternOp::from_tuple(&t).unwrap();
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_override_round_trip() {
    let pat = PatternOp::Override {
        class_id: "C1".into(),
        method_id: "M1".into(),
    };
    let t = pat.to_tuple();
    assert_eq!(t, vec!["PAT", "OVERRIDE", "C1", "M1"]);
    let back = PatternOp::from_tuple(&t).unwrap();
    assert_eq!(back, pat);
}

#[test]
fn pattern_op_from_invalid_tuple_returns_none() {
    assert_eq!(PatternOp::from_tuple(&[]), None);
    assert_eq!(PatternOp::from_tuple(&["X".into()]), None);
    assert_eq!(PatternOp::from_tuple(&["X".into(), "Y".into()]), None);
    assert_eq!(
        PatternOp::from_tuple(&["NOPE".into(), "CTOR".into(), "C1".into(), "M1".into()]),
        None
    );
    assert_eq!(
        PatternOp::from_tuple(&["PAT".into(), "UNKNOWN".into(), "C1".into(), "M1".into()]),
        None
    );
}

#[test]
fn pattern_op_name() {
    let c = PatternOp::Constructor { class_id: "C1".into(), method_id: "M1".into(), deps: vec![] };
    assert_eq!(c.name(), "CTOR");
    let g = PatternOp::Getter { class_id: "C1".into(), method_id: "M1".into(), property: "x".into() };
    assert_eq!(g.name(), "GETTER");
    let s = PatternOp::Setter { class_id: "C1".into(), method_id: "M1".into(), property: "x".into() };
    assert_eq!(s.name(), "SETTER");
    let o = PatternOp::Observable { class_id: "C1".into(), method_id: "M1".into(), return_type: "$P".into() };
    assert_eq!(o.name(), "OBSERVABLE");
    let p = PatternOp::Promise { class_id: "C1".into(), method_id: "M1".into(), return_type: "$P".into() };
    assert_eq!(p.name(), "PROMISE");
    let ov = PatternOp::Override { class_id: "C1".into(), method_id: "M1".into() };
    assert_eq!(ov.name(), "OVERRIDE");
    let ec = PatternOp::EmptyConstructor { class_id: "C1".into(), method_id: "M1".into() };
    assert_eq!(ec.name(), "EMPTY_CTOR");
}

// ── CTOR pattern ──────────────────────────────────────────────

#[test]
fn compress_constructor_with_injects() {
    let ops = vec![
        defclass("C1", "Service"),
        defmethod("C1", "M1", "constructor"),
        param("M1", "P1", "$s", "dep1"),
        param("M1", "P2", "$s", "dep2"),
        ret("M1", "$v"),
        injects("C1", &["S1", "S2"]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, stats) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    let ctor = &pats[0];
    assert!(matches!(ctor, PatternOp::Constructor { .. }));
    if let PatternOp::Constructor { class_id, method_id, deps } = ctor {
        assert_eq!(class_id, "C1");
        assert_eq!(method_id, "M1");
        assert_eq!(deps, &vec!["S1".to_string(), "S2".to_string()]);
    }
    assert_eq!(stats.source_ops, 6);
    assert_eq!(stats.output_ops, 1);
    assert!((stats.ratio - 6.0).abs() < 0.001);
}

#[test]
fn compress_constructor_with_params_no_injects() {
    let ops = vec![
        defmethod("C1", "M1", "constructor"),
        param("M1", "P1", "$s", "name"),
        ret("M1", "$v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    let ctor = &pats[0];
    assert!(matches!(ctor, PatternOp::Constructor { .. }));
    if let PatternOp::Constructor { deps, .. } = ctor {
        assert!(deps.is_empty());
    }
}

#[test]
fn compress_constructor_with_injects_no_params() {
    let ops = vec![
        defmethod("C1", "M1", "constructor"),
        ret("M1", "$v"),
        injects("C1", &["S1"]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Constructor { deps, .. } if !deps.is_empty()));
}

#[test]
fn compress_new_constructor_name() {
    let ops = vec![
        defmethod("C1", "M1", "new"),
        param("M1", "P1", "$s", "x"),
        ret("M1", "$v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Constructor { .. }));
}

// ── Empty CTOR ────────────────────────────────────────────────

#[test]
fn compress_empty_constructor() {
    let ops = vec![
        defmethod("C1", "M1", "constructor"),
        ret("M1", "$v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::EmptyConstructor { .. }));
}

// ── Observable ────────────────────────────────────────────────

#[test]
fn compress_observable_with_async() {
    let ops = vec![
        defmethod("C1", "M1", "fetchData"),
        ret("M1", "$P"),
        flags("M1", &["ASYNC"]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    let obs = &pats[0];
    assert!(matches!(obs, PatternOp::Observable { .. }));
    if let PatternOp::Observable { return_type, .. } = obs {
        assert_eq!(return_type, "$P");
    }
}

#[test]
fn compress_observable_with_observable_type() {
    let ops = vec![
        defmethod("C1", "M1", "stream"),
        ret("M1", "Observable"),
        flags("M1", &["ASYNC"]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Observable { .. }));
}

#[test]
fn compress_observable_requires_async_flag() {
    let ops = vec![
        defmethod("C1", "M1", "fetchData"),
        ret("M1", "$P"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Promise { .. }));
}

// ── Promise ───────────────────────────────────────────────────

#[test]
fn compress_promise_without_async() {
    let ops = vec![
        defmethod("C1", "M1", "fetchData"),
        ret("M1", "$P"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Promise { .. }));
}

#[test]
fn compress_does_not_match_promise_for_non_promise_type() {
    let ops = vec![
        defmethod("C1", "M1", "doWork"),
        ret("M1", "$v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert!(pats.is_empty());
}

// ── Getter/Setter ─────────────────────────────────────────────

#[test]
fn compress_getter() {
    let ops = vec![
        defmethod("C1", "M1", "get fullName"),
        ret("M1", "$s"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    let g = &pats[0];
    assert!(matches!(g, PatternOp::Getter { .. }));
    if let PatternOp::Getter { property, .. } = g {
        assert_eq!(property, "fullName");
    }
}

#[test]
fn compress_getter_no_return() {
    let ops = vec![defmethod("C1", "M1", "get name")];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Getter { .. }));
}

#[test]
fn compress_setter() {
    let ops = vec![
        defmethod("C1", "M1", "set fullName"),
        param("M1", "P1", "$s", "value"),
        ret("M1", "$v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    let s = &pats[0];
    assert!(matches!(s, PatternOp::Setter { .. }));
    if let PatternOp::Setter { property, .. } = s {
        assert_eq!(property, "fullName");
    }
}

#[test]
fn compress_setter_no_param_no_return() {
    let ops = vec![defmethod("C1", "M1", "set x")];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Setter { .. }));
}

#[test]
fn get_set_case_insensitive() {
    let ops = vec![defmethod("C1", "M1", "GET fullName")];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Getter { .. }));
}

// ── Override ──────────────────────────────────────────────────

#[test]
fn compress_override() {
    let ops = vec![
        defmethod("C1", "M1", "toString"),
        flags("M1", &["OVERRIDE"]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
    assert!(matches!(&pats[0], PatternOp::Override { .. }));
}

#[test]
fn override_requires_flag() {
    let ops = vec![defmethod("C1", "M1", "toString")];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert!(pats.is_empty());
}

// ── No match / pass-through ───────────────────────────────────

#[test]
fn no_match_returns_empty() {
    let ops = vec![
        defclass("C1", "Foo"),
        defmethod("C1", "M1", "doWork"),
        param("M1", "P1", "$s", "x"),
        ret("M1", "$b"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, stats) = rec.compress(&ops);
    assert!(pats.is_empty());
    assert_eq!(stats.output_ops, 0);
    assert_eq!(stats.ratio, 0.0);
}

#[test]
fn empty_input_returns_empty() {
    let rec = CompressingPatternRecognizer::new();
    let empty: Vec<CoreOp> = Vec::new();
    let (pats, stats) = rec.compress(&empty);
    assert!(pats.is_empty());
    assert_eq!(stats.source_ops, 0);
    assert_eq!(stats.output_ops, 0);
    assert_eq!(stats.ratio, 0.0);
}

#[test]
fn no_match_with_class_def() {
    let ops = vec![defclass("C1", "Foo")];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert!(pats.is_empty());
}

// ── Mixed / compress_merged ───────────────────────────────────

#[test]
fn compress_merged_preserves_passthroughs() {
    let ops = vec![
        defclass("C1", "Service"),
        defmethod("C1", "M1", "constructor"),
        param("M1", "P1", "$s", "dep"),
        ret("M1", "$v"),
        injects("C1", &["S1"]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let merged = rec.compress_merged(&ops);
    let has_class = merged.iter().any(|i|
        matches!(i, CompressedItem::Passthrough(CoreOp::DefClass(_, _))));
    let has_ctor = merged.iter().any(|i|
        matches!(i, CompressedItem::Pattern(PatternOp::Constructor { .. })));
    assert!(has_class, "DEF_C should pass through");
    assert!(has_ctor, "constructor should compress to Pattern");
}

#[test]
fn compress_merged_no_patterns_means_all_passthrough() {
    let ops = vec![
        defclass("C1", "Foo"),
        defmethod("C1", "M1", "doWork"),
        ret("M1", "$b"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let merged = rec.compress_merged(&ops);
    assert!(merged.iter().all(|i| matches!(i, CompressedItem::Passthrough(_))));
    assert_eq!(merged.len(), 3);
}

#[test]
fn compress_multiple_patterns_in_stream() {
    let ops = vec![
        defmethod("C1", "M1", "get name"),
        defmethod("C1", "M2", "constructor"),
        param("M2", "P1", "$s", "x"),
        ret("M2", "$v"),
        defmethod("C1", "M3", "set name"),
        param("M3", "P1", "$s", "v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 3);
    let names: Vec<&str> = pats.iter().map(|p| p.name()).collect();
    assert!(names.contains(&"GETTER"));
    assert!(names.contains(&"CTOR"));
    assert!(names.contains(&"SETTER"));
}

#[test]
fn pattern_consumed_counts() {
    let c = PatternOp::Constructor { class_id: "C1".into(), method_id: "M1".into(), deps: vec![] };
    assert!(c.consumed() >= 3);
    let g = PatternOp::Getter { class_id: "C1".into(), method_id: "M1".into(), property: "x".into() };
    assert!(g.consumed() >= 2);
    let o = PatternOp::Override { class_id: "C1".into(), method_id: "M1".into() };
    assert!(o.consumed() >= 2);
}

#[test]
fn recognizer_default_impl() {
    let rec: CompressingPatternRecognizer = Default::default();
    let ops = vec![defmethod("C1", "M1", "get x")];
    let (pats, _) = rec.compress(&ops);
    assert_eq!(pats.len(), 1);
}
