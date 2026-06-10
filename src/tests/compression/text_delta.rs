// Tests for Phase IV (Idea #12): Delta-Aware Text Compression

use crate::compression::text_delta::{TextDelta, TextDeltaComputer, apply_text_delta};

#[test]
fn test_text_delta_empty() {
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec![],
        dels: vec![],
        mods: vec![],
    };
    assert!(delta.is_empty());
}

#[test]
fn test_text_delta_wire_format_round_trip() {
    let delta = TextDelta {
        file: "α1".into(),
        from: 3,
        to: 4,
        adds: vec!["$ctor C1 M5 $s data".into()],
        dels: vec!["FLAGS M2 IF".into()],
        mods: vec![("$r M1 $b".into(), "$r M1 $v".into())],
    };

    let wire = delta.to_wire_format();
    assert!(wire.starts_with("§Δα1:3:4§"));
    assert!(wire.contains("\n+$ctor C1 M5 $s data"));
    assert!(wire.contains("\n-FLAGS M2 IF"));
    assert!(wire.contains("\n~$r M1 $b→$r M1 $v"));

    let parsed = TextDelta::from_wire_format(&wire).unwrap();
    assert_eq!(parsed.file, "α1");
    assert_eq!(parsed.from, 3);
    assert_eq!(parsed.to, 4);
    assert_eq!(parsed.adds, vec!["$ctor C1 M5 $s data"]);
    assert_eq!(parsed.dels, vec!["FLAGS M2 IF"]);
    assert_eq!(parsed.mods, vec![("$r M1 $b".to_string(), "$r M1 $v".to_string())]);
}

#[test]
fn test_text_delta_wire_format_no_ops() {
    let delta = TextDelta {
        file: "α2".into(),
        from: 1,
        to: 2,
        adds: vec![],
        dels: vec![],
        mods: vec![],
    };
    let wire = delta.to_wire_format();
    assert_eq!(wire, "§Δα2:1:2§");

    let parsed = TextDelta::from_wire_format(&wire).unwrap();
    assert!(parsed.is_empty());
}

#[test]
fn test_text_delta_wire_format_multiple_adds() {
    let delta = TextDelta {
        file: "α3".into(),
        from: 0,
        to: 1,
        adds: vec!["line1".into(), "line2".into(), "line3".into()],
        dels: vec![],
        mods: vec![],
    };
    let wire = delta.to_wire_format();
    let parsed = TextDelta::from_wire_format(&wire).unwrap();
    assert_eq!(parsed.adds.len(), 3);
    assert_eq!(parsed.adds, vec!["line1", "line2", "line3"]);
}

#[test]
fn test_text_delta_invalid_wire_format() {
    assert!(TextDelta::from_wire_format("not a delta").is_none());
    assert!(TextDelta::from_wire_format("§Δbad§").is_none());
    assert!(TextDelta::from_wire_format("").is_none());
}

#[test]
fn test_delta_computer_no_baseline() {
    let mut computer = TextDeltaComputer::new();
    let lines = vec!["line1".into(), "line2".into()];

    // First call: no baseline → returns None, stores snapshot
    let result = computer.compute_and_store("α1", lines.clone());
    assert!(result.is_none());
    assert!(computer.has_baseline("α1"));
    assert_eq!(computer.file_version("α1"), 1);
}

#[test]
fn test_delta_computer_with_baseline() {
    let mut computer = TextDeltaComputer::new();

    // First call: store baseline
    let baseline = vec![
        "$ctor C1 M1 $s payload".into(),
        "$r M1 $b".into(),
        "FLAGS M1 IF".into(),
    ];
    computer.compute_and_store("α1", baseline);
    assert_eq!(computer.file_version("α1"), 1);

    // Second call: modified — should produce delta
    let modified = vec![
        "$ctor C1 M1 $s payload".into(),
        "$r M1 $b".into(),
        // "FLAGS M1 IF" removed
        "$ctor C1 M2 $s data".into(),  // new method
    ];
    let delta = computer.compute_and_store("α1", modified).unwrap();
    assert_eq!(delta.from, 1);
    assert_eq!(delta.to, 2);
    assert!(delta.dels.contains(&"FLAGS M1 IF".to_string()));
    assert!(delta.adds.contains(&"$ctor C1 M2 $s data".to_string()));
}

#[test]
fn test_delta_computer_no_changes() {
    let mut computer = TextDeltaComputer::new();

    let lines = vec!["line1".into(), "line2".into()];
    computer.compute_and_store("α1", lines.clone());

    // Same lines → no delta
    let result = computer.compute_and_store("α1", lines);
    assert!(result.is_none());
}

#[test]
fn test_apply_text_delta_adds() {
    let baseline = vec!["line1".into(), "line2".into()];
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec!["line3".into()],
        dels: vec![],
        mods: vec![],
    };
    let result = apply_text_delta(&baseline, &delta).unwrap();
    assert_eq!(result, vec!["line1", "line2", "line3"]);
}

#[test]
fn test_apply_text_delta_dels() {
    let baseline = vec!["line1".into(), "line2".into(), "line3".into()];
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec![],
        dels: vec!["line2".into()],
        mods: vec![],
    };
    let result = apply_text_delta(&baseline, &delta).unwrap();
    assert_eq!(result, vec!["line1", "line3"]);
}

#[test]
fn test_apply_text_delta_mods() {
    let baseline = vec!["line1".into(), "line2_old".into(), "line3".into()];
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec![],
        dels: vec![],
        mods: vec![("line2_old".into(), "line2_new".into())],
    };
    let result = apply_text_delta(&baseline, &delta).unwrap();
    assert_eq!(result, vec!["line1", "line2_new", "line3"]);
}

#[test]
fn test_apply_text_delta_combined() {
    let baseline = vec![
        "$ctor C1 M1 $s payload".into(),
        "$r M1 $b".into(),
        "FLAGS M1 IF".into(),
    ];
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec!["$ctor C1 M2 $s data".into()],
        dels: vec!["FLAGS M1 IF".into()],
        mods: vec![("$r M1 $b".into(), "$r M1 $v".into())],
    };
    let result = apply_text_delta(&baseline, &delta).unwrap();
    assert_eq!(result, vec![
        "$ctor C1 M1 $s payload",
        "$r M1 $v",
        "$ctor C1 M2 $s data",
    ]);
}

#[test]
fn test_apply_text_delta_missing_line() {
    let baseline = vec!["line1".into()];
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec![],
        dels: vec!["nonexistent".into()],
        mods: vec![],
    };
    let result = apply_text_delta(&baseline, &delta);
    assert!(result.is_err());
}

#[test]
fn test_delta_computer_multiple_files() {
    let mut computer = TextDeltaComputer::new();

    computer.compute_and_store("α1", vec!["a1_line1".into()]);
    computer.compute_and_store("α2", vec!["a2_line1".into()]);

    assert!(computer.has_baseline("α1"));
    assert!(computer.has_baseline("α2"));
    assert_eq!(computer.file_version("α1"), 1);
    assert_eq!(computer.file_version("α2"), 1);

    // Modify α1
    let delta = computer.compute_and_store("α1", vec!["a1_line1".into(), "a1_line2".into()]).unwrap();
    assert_eq!(delta.from, 1);
    assert_eq!(delta.to, 2);
    assert!(delta.adds.contains(&"a1_line2".to_string()));

    // α2 should be unaffected
    assert_eq!(computer.file_version("α2"), 1);
}

#[test]
fn test_wire_format_modification_with_arrow() {
    // Ensure → (U+2192) is properly handled in wire format
    let delta = TextDelta {
        file: "α1".into(),
        from: 1,
        to: 2,
        adds: vec![],
        dels: vec![],
        mods: vec![("old_value".into(), "new_value".into())],
    };
    let wire = delta.to_wire_format();
    assert!(wire.contains("~old_value→new_value"));

    let parsed = TextDelta::from_wire_format(&wire).unwrap();
    assert_eq!(parsed.mods.len(), 1);
    assert_eq!(parsed.mods[0].0, "old_value");
    assert_eq!(parsed.mods[0].1, "new_value");
}