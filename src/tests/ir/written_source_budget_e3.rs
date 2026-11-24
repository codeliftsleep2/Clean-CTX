use super::*;
use serde_json::json;
use std::sync::{
    Barrier,
    atomic::{AtomicUsize, Ordering},
};

fn limits(tasks: usize) -> Limits {
    Limits {
        total: Cost::from_values([100, 100, 100, 100, 100, 100, 100, 100]),
        finalization: Cost::default(),
        deadline: Duration::from_secs(10),
        finalization_time: Duration::ZERO,
        active_tasks: tasks,
    }
}

#[test]
fn e3_every_resource_dimension_refuses_before_work_and_without_partial_debits() {
    for (index, reason) in DIMENSIONS.iter().enumerate() {
        let request = Request::new(limits(1)).unwrap();
        let mut cost = [0; 8];
        cost[index] = 101;
        let called = AtomicUsize::new(0);
        let result = request.run_optional(Cost::from_values(cost), |_| {
            called.fetch_add(1, Ordering::SeqCst);
        });
        assert!(matches!(result,Err(actual) if actual==*reason));
        assert_eq!(called.load(Ordering::SeqCst), 0);
        assert_eq!(request.snapshot().used, Cost::default());
    }
    let mut invalid = limits(1);
    invalid.finalization.units = 101;
    assert!(Request::new(invalid).is_err());
    invalid = limits(0);
    assert!(Request::new(invalid).is_err());
}

#[test]
fn e3_nested_requests_share_cumulative_work_and_release_retained_ownership() {
    let request = Request::new(limits(2)).unwrap();
    let outer = request
        .run_optional(
            Cost {
                source_bytes: 60,
                retained_bytes: 40,
                ..Cost::default()
            },
            |nested| {
                let clone = nested.clone();
                assert!(matches!(
                    clone.reserve_optional(Cost {
                        source_bytes: 41,
                        ..Cost::default()
                    }),
                    Err(Stop::SourceBytes)
                ));
                let inside = clone
                    .run_optional(
                        Cost {
                            source_bytes: 40,
                            retained_bytes: 30,
                            ..Cost::default()
                        },
                        |_| vec![0u8; 30],
                    )
                    .unwrap();
                assert_eq!(clone.snapshot().used.retained_bytes, 70);
                drop(inside);
                assert_eq!(clone.snapshot().used.retained_bytes, 40);
                vec![0u8; 40]
            },
        )
        .unwrap();
    assert_eq!(outer.value().len(), 40);
    assert_eq!(request.snapshot().used.source_bytes, 100);
    assert_eq!(request.snapshot().used.retained_bytes, 40);
    drop(outer);
    assert_eq!(request.snapshot().used.retained_bytes, 0);
    assert_eq!(request.snapshot().retained_peak, 70);
    assert_eq!(request.snapshot().active_tasks, 0);
}

#[test]
fn e3_finalization_has_protected_capacity_but_not_unlimited_time_or_bytes() {
    let mut l = limits(1);
    l.finalization.validation_bytes = 30;
    l.finalization.response_bytes = 20;
    l.finalization_time = Duration::from_secs(2);
    let request = Request::new(l).unwrap();
    drop(
        request
            .reserve_optional(Cost {
                validation_bytes: 70,
                response_bytes: 80,
                ..Cost::default()
            })
            .unwrap(),
    );
    assert!(matches!(
        request.reserve_optional(Cost {
            response_bytes: 1,
            ..Cost::default()
        }),
        Err(Stop::ResponseBytes)
    ));
    request.cancel_optional();
    assert_eq!(request.checkpoint(), Err(Stop::Cancelled));
    drop(
        request
            .reserve_finalization(Cost {
                validation_bytes: 30,
                response_bytes: 20,
                ..Cost::default()
            })
            .unwrap(),
    );
    assert!(matches!(
        request.reserve_finalization(Cost {
            response_bytes: 1,
            ..Cost::default()
        }),
        Err(Stop::ResponseBytes)
    ));
    assert_eq!(request.snapshot().used.validation_bytes, 100);
    let clock = Request::new(l).unwrap();
    assert_eq!(
        clock.check_at(false, Duration::from_secs(8)),
        Err(Stop::Deadline)
    );
    assert_eq!(clock.check_at(true, Duration::from_secs(8)), Ok(()));
    assert_eq!(
        clock.check_at(true, Duration::from_secs(10)),
        Err(Stop::Deadline)
    );
}

#[test]
fn e3_concurrency_reservations_and_panic_cleanup_are_request_local() {
    let request = Request::new(limits(1)).unwrap();
    let held = request
        .reserve_optional(Cost {
            retained_bytes: 10,
            ..Cost::default()
        })
        .unwrap();
    let other = request.clone();
    std::thread::spawn(move || {
        assert!(matches!(
            other.reserve_optional(Cost::default()),
            Err(Stop::ActiveTasks)
        ))
    })
    .join()
    .unwrap();
    drop(held);
    let result = std::panic::catch_unwind(|| {
        let _ = request.run_optional(
            Cost {
                retained_bytes: 50,
                ..Cost::default()
            },
            |_| panic!("work failure"),
        );
    });
    assert!(result.is_err());
    assert_eq!(request.snapshot().active_tasks, 0);
    assert_eq!(request.snapshot().used.retained_bytes, 0);
    assert_eq!(request.snapshot().active_peak, 1);
}

#[test]
fn e3_cancellation_preserves_observations_but_never_bypasses_freshness() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app.ts");
    std::fs::write(&app, "const x=[{path:'a',resolve:{user:U}}];").unwrap();
    let candidate = Candidate::capture(&[root.path().to_path_buf()], &app).unwrap();
    let mut l = limits(1);
    l.total.validation_bytes = 1024;
    l.total.validation_checks = 1024;
    l.finalization.validation_bytes = 1024;
    l.finalization.validation_checks = 1024;
    let request = Request::new(l).unwrap();
    request.cancel_optional();
    let admitted = request.admit(candidate).unwrap();
    let admitted = admitted.value().as_ref().unwrap();
    assert_eq!(admitted.observations().len(), 1);
    assert!(admitted.is_current());
    let candidate = Candidate::capture(&[root.path().to_path_buf()], &app).unwrap();
    std::fs::write(&app, "const x=[];").unwrap();
    let fresh = Request::new(l).unwrap();
    fresh.cancel_optional();
    assert!(matches!(
        fresh.admit(candidate).unwrap().value(),
        Err(Refusal::StaleSource)
    ));
}

#[test]
fn e3_request_worker_and_cancellation_measurements() {
    let Ok(output) = std::env::var("OPS_E3_OUTPUT") else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app.ts");
    std::fs::write(
        &app,
        "const x=[{path:'users',resolve:{user:UserResolver}}];",
    )
    .unwrap();
    let mut data = Vec::new();
    for workers in [1, 2] {
        let mut l = limits(workers);
        l.total = Cost {
            source_bytes: 1024,
            support_bytes: 65536,
            retained_bytes: 32768,
            units: 9,
            edges: 8,
            validation_bytes: 1024,
            validation_checks: 100,
            response_bytes: 1024,
        };
        l.finalization = Cost {
            validation_bytes: 1024,
            validation_checks: 100,
            response_bytes: 1024,
            ..Cost::default()
        };
        let request = Request::new(l).unwrap();
        // The declared 8 KiB envelope covers the small observation/proof result;
        // it is an explicit reservation, not measured allocator interception.
        let captured = request
            .run_optional(
                Cost {
                    source_bytes: std::fs::metadata(&app).unwrap().len(),
                    retained_bytes: 8192,
                    units: 1,
                    ..Cost::default()
                },
                |_| Candidate::capture(&[root.path().to_path_buf()], &app).unwrap(),
            )
            .unwrap();
        let admitted = captured.map(|candidate| request.admit(candidate).unwrap());
        let queue = AtomicUsize::new(0);
        let completed = AtomicUsize::new(0);
        let barrier = Barrier::new(workers);
        std::thread::scope(|scope| {
            for _ in 0..workers {
                let request = &request;
                let queue = &queue;
                let completed = &completed;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    loop {
                        let _id = queue.fetch_add(1, Ordering::SeqCst);
                        let result = request.run_optional(
                            Cost {
                                support_bytes: 8192,
                                retained_bytes: 8192,
                                units: 1,
                                edges: 1,
                                ..Cost::default()
                            },
                            |same| {
                                let buffer = vec![7u8; 8192];
                                assert_eq!(same.snapshot().used.support_bytes % 8192, 0);
                                std::thread::sleep(Duration::from_millis(2));
                                buffer
                            },
                        );
                        match result {
                            Ok(value) => {
                                assert_eq!(value.value().len(), 8192);
                                completed.fetch_add(1, Ordering::SeqCst);
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
        });
        request.cancel_optional();
        assert!(
            *request
                .validate(admitted.value().value().as_ref().unwrap())
                .unwrap()
                .value()
        );
        let response = request
            .reserve_finalization(Cost {
                response_bytes: 128,
                ..Cost::default()
            })
            .unwrap();
        drop(response);
        let s = request.snapshot();
        assert_eq!(completed.load(Ordering::SeqCst), 8);
        assert_eq!(s.used.support_bytes, 65536);
        assert_eq!(s.used.retained_bytes, 8192);
        assert!(s.active_peak <= workers);
        data.push(json!({"workers":workers,"completed_units":completed.load(Ordering::SeqCst),
            "elapsed_ms":s.elapsed.as_secs_f64()*1000.0,"active_peak":s.active_peak,
            "reserved_retained_peak":s.retained_peak,"support_bytes":s.used.support_bytes,
            "validation_bytes":s.used.validation_bytes,"validation_checks":s.used.validation_checks,
            "response_bytes":s.used.response_bytes,"observations_survived":true,
            "actual_allocation_note":"Buffers equal reservations; allocator/runtime/E2 proof not intercepted"}));
        drop(admitted);
        assert_eq!(request.snapshot().used.retained_bytes, 0);
    }
    let mut l = limits(1);
    l.deadline = Duration::from_millis(10);
    let request = Request::new(l).unwrap();
    let work = request
        .run_optional(Cost::default(), |_| {
            std::thread::sleep(Duration::from_millis(25))
        })
        .unwrap();
    assert_eq!(work.completed_after_stop, Some(Stop::Deadline));
    let s = request.snapshot();
    data.push(json!({"kind":"non_interruptible_quantum","deadline_ms":10,"elapsed_ms":s.elapsed.as_secs_f64()*1000.0,
        "overshoot_ms":s.deadline_overshoot.as_secs_f64()*1000.0,"late_result_reported":true}));
    let request = Request::new(limits(1)).unwrap();
    let entered = Barrier::new(2);
    let cancelled = Barrier::new(2);
    std::thread::scope(|scope| {
        let r = &request;
        let entered = &entered;
        let cancelled = &cancelled;
        let thread = scope.spawn(move || {
            r.run_optional(Cost::default(), |_| {
                entered.wait();
                cancelled.wait();
                std::thread::sleep(Duration::from_millis(5));
            })
            .unwrap()
        });
        entered.wait();
        let start = Instant::now();
        request.cancel_optional();
        cancelled.wait();
        let result = thread.join().unwrap();
        assert_eq!(result.completed_after_stop, Some(Stop::Cancelled));
        data.push(json!({"kind":"inflight_cancel","cancel_to_return_ms":start.elapsed().as_secs_f64()*1000.0,
            "late_result_reported":true,"further_optional_work_refused":request.checkpoint()==Err(Stop::Cancelled)}));
    });
    std::fs::write(
        std::path::Path::new(&output).join("measurements.json"),
        serde_json::to_vec_pretty(&data).unwrap(),
    )
    .unwrap();
}
