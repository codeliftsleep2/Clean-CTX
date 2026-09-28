//! Bounded measurement harness for the compilation-scoped meta-layer pass.
//!
//! This is diagnostic evidence, not a correctness test or release gate. Run:
//!
//! `cargo run --release --all-features --example meta_layer_benchmark`

use std::hint::black_box;
use std::time::Instant;

use clean_ctx::compression::Fidelity;
use clean_ctx::config::{CleanCtxConfig, MetaLayerConfig};
use clean_ctx::ir::compiler::IRCompiler;

const WARMUP_ITERATIONS: usize = 10;
const MEASURED_ITERATIONS: usize = 100;

struct Case {
    name: &'static str,
    extension: &'static str,
    path: &'static str,
    framework: &'static str,
    source: &'static str,
}

fn disabled_config(framework: &str) -> CleanCtxConfig {
    let mut config = CleanCtxConfig::default();
    config.meta_layers.insert(
        framework.to_string(),
        MetaLayerConfig {
            enabled: false,
            ..MetaLayerConfig::default()
        },
    );
    config
}

fn compile(case: &Case, enabled: bool) {
    let (language, query) = clean_ctx::compression::language::language_for_extension(case.extension)
        .expect("benchmark language feature must be enabled");
    let mut compiler = IRCompiler::new();
    if !enabled {
        compiler.set_config(disabled_config(case.framework));
    }
    let ir = compiler
        .compile_focused(
            black_box(case.source),
            "benchmark",
            Some(case.path),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("representative source must compile");
    black_box(ir.instructions.len() + compiler.semantic_edges.len());
}

fn timed_compile(case: &Case, enabled: bool) -> f64 {
    let started = Instant::now();
    compile(case, enabled);
    started.elapsed().as_secs_f64() * 1_000_000.0
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() & 1 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn report(case: &Case) {
    for iteration in 0..WARMUP_ITERATIONS {
        compile(case, iteration & 1 == 0);
        compile(case, iteration & 1 != 0);
    }

    let mut enabled_samples = Vec::with_capacity(MEASURED_ITERATIONS);
    let mut disabled_samples = Vec::with_capacity(MEASURED_ITERATIONS);
    let mut paired_deltas = Vec::with_capacity(MEASURED_ITERATIONS);
    for iteration in 0..MEASURED_ITERATIONS {
        let (enabled, disabled) = if iteration & 1 == 0 {
            (timed_compile(case, true), timed_compile(case, false))
        } else {
            let disabled = timed_compile(case, false);
            (timed_compile(case, true), disabled)
        };
        enabled_samples.push(enabled);
        disabled_samples.push(disabled);
        paired_deltas.push(enabled - disabled);
    }

    let enabled_us = median(&mut enabled_samples);
    let disabled_us = median(&mut disabled_samples);
    let meta_us = median(&mut paired_deltas);
    let share = if enabled_us == 0.0 {
        0.0
    } else {
        meta_us / enabled_us * 100.0
    };
    println!(
        "{:<10} median enabled={:>9.2} us  disabled={:>9.2} us  paired delta={:>9.2} us ({:>6.2}%)",
        case.name, enabled_us, disabled_us, meta_us, share
    );
}

fn main() {
    let cases: &[Case] = &[
    #[cfg(feature = "angular")]
    Case {
        name: "Angular",
        extension: "ts",
        path: "C:/benchmark/app.component.ts",
        framework: "angular",
        source: r#"
import { Component, computed, inject, signal } from '@angular/core';
import { FormBuilder, Validators } from '@angular/forms';
import { Store, createAction, createReducer, createSelector, on } from '@ngrx/store';
import { Observable, map, switchMap } from 'rxjs';
@Component({ selector: 'app-root', template: '<button (click)="save()">Save</button>' })
export class AppComponent {
  private readonly store = inject(Store);
  private readonly fb = inject(FormBuilder);
  readonly count = signal(0);
  readonly doubled = computed(() => this.count() * 2);
  readonly form = this.fb.group({ name: ['', Validators.required] });
  readonly result$: Observable<string> = this.store.select(selectName).pipe(map(x => x.trim()));
  save() { this.store.dispatch(saveUser({ name: this.form.value.name })); }
}
export const saveUser = createAction('[User] Save');
export const reducer = createReducer({}, on(saveUser, state => state));
export const selectName = createSelector(selectUser, user => user.name);
function selectUser(state: any) { return state.user; }
"#,
    },

    #[cfg(feature = "dotnet")]
    Case {
        name: ".NET",
        extension: "cs",
        path: "C:/benchmark/UsersController.cs",
        framework: "dotnet",
        source: r#"
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
[ApiController]
[Route("api/[controller]")]
public class UsersController : ControllerBase {
    private readonly AppDbContext _db;
    public UsersController(AppDbContext db) { _db = db; }
    [HttpGet("{id}")]
    public async Task<ActionResult<User>> Get(int id) {
        var user = await _db.Users.FirstOrDefaultAsync(x => x.Id == id);
        return user is null ? NotFound() : Ok(user);
    }
    [HttpPost]
    public async Task<ActionResult<User>> Create(User user) {
        _db.Users.Add(user);
        await _db.SaveChangesAsync();
        return CreatedAtAction(nameof(Get), new { id = user.Id }, user);
    }
}
public class AppDbContext : DbContext { public DbSet<User> Users { get; set; } }
public record User(int Id, string Name);
"#,
    },

    #[cfg(feature = "spring_boot")]
    Case {
        name: "Spring",
        extension: "java",
        path: "C:/benchmark/UsersController.java",
        framework: "spring_boot",
        source: r#"
package example.users;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
@RestController
@RequestMapping("/api/users")
class UsersController {
    private final UserService service;
    UsersController(UserService service) { this.service = service; }
    @GetMapping("/{id}")
    ResponseEntity<User> get(@PathVariable long id) {
        return service.find(id).map(ResponseEntity::ok).orElseGet(() -> ResponseEntity.notFound().build());
    }
    @PostMapping
    ResponseEntity<User> create(@RequestBody User user) {
        return ResponseEntity.ok(service.save(user));
    }
}
@Service
class UserService {
    private final UserRepository repository;
    UserService(UserRepository repository) { this.repository = repository; }
    Optional<User> find(long id) { return repository.findById(id); }
    User save(User user) { return repository.save(user); }
}
interface UserRepository extends JpaRepository<User, Long> {}
record User(long id, String name) {}
"#,
    },
    ];

    println!("Meta-layer production-path benchmark");
    println!("warmup={WARMUP_ITERATIONS}, measured={MEASURED_ITERATIONS} per mode");
    println!("modes alternate per sample; paired median reduces run-order drift");
    println!("delta remains directional evidence; repeat runs before drawing conclusions\n");
    for case in cases {
        report(case);
    }
}
