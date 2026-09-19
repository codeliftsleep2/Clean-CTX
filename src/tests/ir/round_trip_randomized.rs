use super::*;

// ── 6. Randomized Property Tests ────────────────────────────────
//
// These tests generate random IRs with all variant types and verify
// that encode → decode → encode → decode produces identical results.
// This catches edge cases that hand-written tests might miss.

/// Generate a random CoreOp for property testing.
fn random_op(rng: &mut impl FnMut() -> u64) -> CoreOp {
    let variant = rng() % 22;
    match variant {
        0 => CoreOp::DefClass(format!("C{}", rng() % 10), format!("Class{}", rng() % 100)),
        1 => CoreOp::DefMethod(
            format!("C{}", rng() % 10),
            format!("M{}", rng() % 10),
            format!("method{}", rng() % 100),
        ),
        2 => CoreOp::DefField(
            format!("C{}", rng() % 10),
            format!("F{}", rng() % 10),
            format!("field{}", rng() % 100),
        ),
        3 => CoreOp::DefInterface(format!("I{}", rng() % 10), format!("Iface{}", rng() % 100)),
        4 => CoreOp::Param(
            format!("M{}", rng() % 10),
            format!("P{}", rng() % 10),
            match rng() % 4 {
                0 => "$s",
                1 => "$n",
                2 => "$b",
                _ => "$v",
            }
            .to_string(),
            format!("param{}", rng() % 100),
        ),
        5 => CoreOp::Return(
            format!("M{}", rng() % 10),
            match rng() % 4 {
                0 => "$s",
                1 => "$n",
                2 => "$b",
                _ => "$v",
            }
            .to_string(),
        ),
        6 => CoreOp::FieldType(
            format!("F{}", rng() % 10),
            match rng() % 4 {
                0 => "$s",
                1 => "$n",
                2 => "$b",
                _ => "$v",
            }
            .to_string(),
        ),
        7 => CoreOp::Flags(
            format!("M{}", rng() % 10),
            vec![
                match rng() % 4 {
                    0 => "IF",
                    1 => "LOOP",
                    2 => "ASYNC",
                    _ => "RET",
                }
                .to_string(),
            ],
        ),
        8 => CoreOp::ClassFlags(
            format!("C{}", rng() % 10),
            vec![
                match rng() % 3 {
                    0 => "EXPORT",
                    1 => "ABSTRACT",
                    _ => "STATIC",
                }
                .to_string(),
            ],
        ),
        9 => CoreOp::Extends(format!("C{}", rng() % 10), format!("C{}", rng() % 10)),
        10 => CoreOp::Implements(format!("C{}", rng() % 10), format!("I{}", rng() % 10)),
        11 => CoreOp::Injects(
            format!("C{}", rng() % 10),
            vec![format!("Dep{}", rng() % 10)],
        ),
        12 => CoreOp::Import(
            format!("IM{}", rng() % 10),
            format!("module{}", rng() % 10),
            format!("export{}", rng() % 10),
        ),
        13 => CoreOp::TypeAlias(format!("T{}", rng() % 10), format!("Type{}", rng() % 10)),
        14 => CoreOp::Pattern(
            format!("PAT{}", rng() % 10),
            vec![format!("arg{}", rng() % 10)],
        ),
        // R-43a: Execution Semantics
        15 => CoreOp::DataFlow(
            format!("M{}", rng() % 10),
            match rng() % 2 {
                0 => "reads".to_string(),
                _ => "writes".to_string(),
            },
            format!("target{}", rng() % 10),
        ),
        16 => CoreOp::ControlFlow(
            format!("M{}", rng() % 10),
            match rng() % 6 {
                0 => "if",
                1 => "loop",
                2 => "match",
                3 => "try",
                4 => "await",
                _ => "return",
            }
            .to_string(),
            format!("expr{}", rng() % 10),
        ),
        17 => CoreOp::SideEffect(
            format!("M{}", rng() % 10),
            match rng() % 5 {
                0 => "pure",
                1 => "io",
                2 => "mutation",
                3 => "async",
                _ => "transaction",
            }
            .to_string(),
        ),
        18 => CoreOp::ExecutionContext(
            format!("M{}", rng() % 10),
            match rng() % 5 {
                0 => "sync",
                1 => "async",
                2 => "thread_bound",
                3 => "transaction_scope",
                _ => "realtime",
            }
            .to_string(),
        ),
        19 => CoreOp::Body(
            format!("M{}", rng() % 10),
            format!(
                "{{
  let x = {};
}}",
                rng() % 100
            ),
            Some(rng() % 4096),
            Some((rng() % 4096) + 128),
        ),
        20 => CoreOp::MethodModifiers(
            format!("M{}", rng() % 10),
            vec![match rng() % 4 {
                0 => DeclarationModifier::Async,
                1 => DeclarationModifier::Generator,
                2 => DeclarationModifier::Static,
                _ => DeclarationModifier::Unsafe,
            }],
        ),
        21 => CoreOp::ClassModifiers(
            format!("C{}", rng() % 10),
            vec![match rng() % 4 {
                0 => DeclarationModifier::Export,
                1 => DeclarationModifier::Private,
                2 => DeclarationModifier::Protected,
                _ => DeclarationModifier::Abstract,
            }],
        ),
        _ => unreachable!(),
    }
}

/// Generate a random CompiledIR with a random number of instructions.
fn random_ir(rng: &mut impl FnMut() -> u64) -> CompiledIR {
    let count = (rng() % 20 + 1) as usize; // 1-20 instructions
    let mut instructions = Vec::with_capacity(count);
    for _ in 0..count {
        instructions.push(random_op(rng));
    }
    CompiledIR {
        file_id: format!("file{}.ts", rng() % 100),
        version: rng() % 100,
        instructions,
    }
}

/// A simple deterministic RNG for property testing.
/// Uses a linear congruential generator with wrapping arithmetic.
fn make_rng(seed: u64) -> impl FnMut() -> u64 {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state >> 33 // Use high bits for better distribution
    }
}

/// Property test: Named wire format round-trip with random IRs.
/// Runs 100 iterations with different random seeds.
#[test]
fn property_named_wire_round_trip() {
    for seed in 0..100 {
        let mut rng = make_rng(seed);
        let original = random_ir(&mut rng);
        let wire = ir_to_wire(&original);
        let restored = wire_to_ir(&wire).unwrap_or_else(|e| {
            panic!("seed {}: named wire decode failed: {}", seed, e);
        });
        assert_eq!(
            restored.instructions.len(),
            original.instructions.len(),
            "seed {}: instruction count mismatch after named wire round-trip",
            seed
        );
        for (i, (a, b)) in restored
            .instructions
            .iter()
            .zip(original.instructions.iter())
            .enumerate()
        {
            assert_eq!(a, b, "seed {}: instruction mismatch at index {}", seed, i);
        }
    }
}

/// Property test: Binary wire format round-trip with random IRs.
/// Runs 100 iterations with different random seeds.
/// Note: Binary wire drops structural parent IDs (class_id, etc.) — we verify
/// opcode match and data field preservation instead of full equality.
#[test]
fn property_binary_wire_round_trip() {
    for seed in 0..100 {
        let mut rng = make_rng(seed);
        let original = random_ir(&mut rng);
        let bytes = encode(&original);
        let restored = decode(&bytes).unwrap_or_else(|e| {
            panic!("seed {}: binary wire decode failed: {}", seed, e);
        });
        assert_eq!(
            restored.instructions.len(),
            original.instructions.len(),
            "seed {}: instruction count mismatch after binary wire round-trip",
            seed
        );
        for (i, (a, b)) in restored
            .instructions
            .iter()
            .zip(original.instructions.iter())
            .enumerate()
        {
            // Binary wire uses empty string for structural parent IDs
            match (a, b) {
                (CoreOp::DefClass(_, _), CoreOp::DefClass(_, _))
                | (CoreOp::DefMethod(_, _, _), CoreOp::DefMethod(_, _, _))
                | (CoreOp::DefField(_, _, _), CoreOp::DefField(_, _, _))
                | (CoreOp::DefInterface(_, _), CoreOp::DefInterface(_, _))
                | (CoreOp::Extends(_, _), CoreOp::Extends(_, _))
                | (CoreOp::Implements(_, _), CoreOp::Implements(_, _))
                | (CoreOp::Import(_, _, _), CoreOp::Import(_, _, _))
                | (CoreOp::TypeAlias(_, _), CoreOp::TypeAlias(_, _)) => {
                    // Just verify opcode matches
                    assert_eq!(
                        crate::ir::wire::op_to_tuple(a)[0],
                        crate::ir::wire::op_to_tuple(b)[0],
                        "seed {}: binary wire opcode mismatch at index {}",
                        seed,
                        i
                    );
                }
                // Data-preserving ops
                (CoreOp::Body(..), CoreOp::Body(..))
                | (CoreOp::DataFlow(..), CoreOp::DataFlow(..))
                | (CoreOp::ControlFlow(..), CoreOp::ControlFlow(..))
                | (CoreOp::SideEffect(..), CoreOp::SideEffect(..))
                | (CoreOp::ExecutionContext(..), CoreOp::ExecutionContext(..))
                | (CoreOp::Param(..), CoreOp::Param(..))
                | (CoreOp::Return(..), CoreOp::Return(..))
                | (CoreOp::FieldType(..), CoreOp::FieldType(..))
                | (CoreOp::MethodModifiers(..), CoreOp::MethodModifiers(..))
                | (CoreOp::ClassModifiers(..), CoreOp::ClassModifiers(..))
                | (CoreOp::Flags(..), CoreOp::Flags(..))
                | (CoreOp::ClassFlags(..), CoreOp::ClassFlags(..))
                | (CoreOp::Injects(..), CoreOp::Injects(..))
                | (CoreOp::Pattern(..), CoreOp::Pattern(..)) => {
                    assert_eq!(
                        a, b,
                        "seed {}: binary wire instruction mismatch at index {}",
                        seed, i
                    );
                }
                _ => panic!("seed {}: variant mismatch at index {}", seed, i),
            }
        }
    }
}

/// Property test: Double encode → decode stability.
/// Verifies that encode → decode → encode → decode produces identical results.
#[test]
fn property_double_encode_stability() {
    for seed in 0..50 {
        let mut rng = make_rng(seed);
        let original = random_ir(&mut rng);

        // Named wire: encode → decode → encode → decode
        let wire1 = ir_to_wire(&original);
        let decoded1 = wire_to_ir(&wire1).unwrap();
        let wire2 = ir_to_wire(&decoded1);
        let decoded2 = wire_to_ir(&wire2).unwrap();
        assert_ir_eq(&decoded1, &decoded2);

        // Binary wire: encode → decode → encode → decode
        let bytes1 = encode(&original);
        let decoded_bin1 = decode(&bytes1).unwrap();
        let bytes2 = encode(&decoded_bin1);
        let decoded_bin2 = decode(&bytes2).unwrap();
        assert_eq!(
            decoded_bin1.instructions.len(),
            decoded_bin2.instructions.len()
        );
        for (i, (a, b)) in decoded_bin1
            .instructions
            .iter()
            .zip(decoded_bin2.instructions.iter())
            .enumerate()
        {
            assert_eq!(
                a, b,
                "seed {}: binary double-encode mismatch at index {}",
                seed, i
            );
        }
    }
}
