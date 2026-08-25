// examples/apply_edit_comparison.rs
//
// Measures the ACTUAL token delta of the `apply_edit` write path vs. the
// read→edit convention, across the 50-edit simulation categories — the
// write-side numbers this plan targets (docs/plans/APPLY_EDIT_PLAN.md
// Phase 5). The read-side savings are already measured by
// `fidelity_comparison.rs`; this benchmark validates the WRITE side the
// same rigorous way before any number is claimed.
//
// The single-edit read→edit convention re-reads the ENTIRE file before
// each write (the whole-file staleness precondition). `apply_edit`
// verifies only the changed unit and ships a tiny operations JSON.
//
// Run with:
//     cargo run --example apply_edit_comparison
//
// Output: per-category cumulative tokens + savings table.

use clean_ctx::analytics::bpe;
use clean_ctx::edit::apply;
use clean_ctx::edit::locate::UnitTable;
use clean_ctx::edit::ops::EditOperation;
use clean_ctx::ir::opcodes::CoreOp;

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditCategory {
    Small,
    Method,
    Structural,
    CrossMethod,
    Refactor,
}

impl EditCategory {
    fn name(self) -> &'static str {
        match self {
            EditCategory::Small => "Small",
            EditCategory::Method => "Method",
            EditCategory::Structural => "Structural",
            EditCategory::CrossMethod => "CrossMethod",
            EditCategory::Refactor => "Refactor",
        }
    }
}

/// A simulated single-unit edit inside the fixture source.
struct Simulated {
    category: EditCategory,
    description: &'static str,
    target: &'static str,
    old_body: &'static str,
    new_body: &'static str,
}

fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source =
        std::fs::read_to_string(manifest_dir.join("src/test_files/UserManagementService.ts"))
            .expect("Cannot read UserManagementService.ts");

    let simulations: Vec<Simulated> = vec![
        Simulated {
            category: EditCategory::Small,
            description: "trim -> trim().toLowerCase()",
            target: "UserService.processOrder",
            old_body: "{\n    return order.trim();\n  }",
            new_body: "{\n    return order.trim().toLowerCase();\n  }",
        },
        Simulated {
            category: EditCategory::Method,
            description: "trim -> trim().toUpperCase()",
            target: "UserService.processOrder",
            old_body: "{\n    return order.trim();\n  }",
            new_body: "{\n    return order.trim().toUpperCase();\n  }",
        },
        Simulated {
            category: EditCategory::Structural,
            description: "count -> return 42",
            target: "UserService.count",
            old_body: "{\n    return 1;\n  }",
            new_body: "{\n    const total = 42;\n    return total;\n  }",
        },
        Simulated {
            category: EditCategory::CrossMethod,
            description: "second edit, same file",
            target: "UserService.count",
            old_body: "{\n    return 42;\n  }",
            new_body: "{\n    return 7;\n  }",
        },
        Simulated {
            category: EditCategory::Refactor,
            description: "hoist computed value",
            target: "UserService.processOrder",
            old_body: "{\n    return order.trim();\n  }",
            new_body: "{\n    const trimmed = order.trim();\n    return trimmed;\n  }",
        },
    ];

    // Build a unit table from the fixture's real spans.
    let body_a = source.find("{\n    return order.trim();").unwrap();
    let end_a = source.find("\n  }\n\n  count()").unwrap() + "\n  }".len();
    let body_b = source.find("{\n    return 1;\n  }").unwrap();
    let end_b = source.find("\n  }\n}\n").unwrap() + "\n  }".len();
    let units = UnitTable::from_instructions(&[
        CoreOp::Body(
            "M1".into(),
            source[body_a..end_a].to_string(),
            Some(body_a as u64),
            Some(end_a as u64),
        ),
        CoreOp::Body(
            "M2".into(),
            source[body_b..end_b].to_string(),
            Some(body_b as u64),
            Some(end_b as u64),
        ),
    ]);

    let tokenizer = bpe();
    let file_tokens = tokenizer.encode_with_special_tokens(&source).len();
    let mut cum_read = 0usize;
    let mut cum_apply = 0usize;
    let mut current = source.clone();
    let mut edits = 0usize;

    println!("apply_edit vs read→edit — WRITE-side token cost (per edit unit)\n");
    println!(
        "{:<11} {:<30} {:>8} {:>10} {:>8}",
        "category", "edit", "read", "apply_edit", "saved"
    );

    for sim in simulations.iter().cycle().take(50) {
        // Read→edit: a full raw read of the file precedes every write.
        let read_cost = file_tokens;

        // apply_edit: build + splice the operations JSON; assert it works.
        let op = EditOperation::ReplaceBody {
            target: sim.target.to_string(),
            expected_old_text: sim.old_body.to_string(),
            new_text: sim.new_body.to_string(),
        };
        let report = apply::apply(&current, &units, std::slice::from_ref(&op))
            .expect("splice should succeed in benchmark");
        let req_json = serde_json::json!({
            "filePath": "UserManagementService.ts",
            "operations": [op],
        });
        let apply_cost = tokenizer
            .encode_with_special_tokens(&serde_json::to_string(&req_json).unwrap())
            .len();
        current = report.new_source;
        edits += 1;

        cum_read += read_cost;
        cum_apply += apply_cost;
        if edits <= 5 || edits % 10 == 0 {
            let saved_pct = read_cost.saturating_sub(apply_cost) * 100 / read_cost.max(1);
            println!(
                "{:<11} {:<30} {:>8} {:>10} {:>7}%",
                sim.category.name(),
                sim.description,
                read_cost,
                apply_cost,
                saved_pct
            );
        }
    }

    let total_saved = cum_read.saturating_sub(cum_apply) * 100 / cum_read.max(1);
    println!("\nCumulative over {edits} simulated single-unit edits:");
    println!("  read→edit convention : {cum_read:>8} tokens");
    println!("  apply_edit           : {cum_apply:>8} tokens");
    println!(
        "  WRITE-side savings   : {total_saved:>7}%  (read-side savings are measured by fidelity_comparison)"
    );
}
