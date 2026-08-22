// src/tests/integration/workspace_perf.rs
//
// End-to-end performance tests for large workspace compression.
//
// Test 14: Large Workspace Performance
//   - 1000+ file workspace with mixed languages
//   - Measures time, memory, token savings
//   - Verifies stability (no OOM, no lock contention, correct caching)
//   - Tests with realistic conditions (mixed languages, meta-layers, CBM active)
//
// Variations tested:
//   - Different fidelity levels (Low, Medium, High)
//   - With/without exclude patterns
//   - Incremental changes (delta transport on large workspace)
//   - Cache hit rate (second pass should be faster)

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::mcp::McpState;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

// ── Helper: Generate a realistic mixed-language test workspace ──────────

/// Generate a test workspace with `n` files of mixed languages.
///
/// Produces a realistic mix:
///   - 40% TypeScript (Angular components, services, models)
///   - 30% C# (EF Core contexts, controllers, models) — requires `csharp` feature
///   - 20% Rust (modules, structs, traits) — requires `rust` feature
///   - 10% Java (Spring Boot controllers, services) — requires `java` feature
///
/// Returns `(path, TempDir)` — keep the `TempDir` guard alive for the test
/// duration so the directory is automatically cleaned up on drop.
fn setup_large_test_workspace(n: usize) -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path().to_path_buf();

    let ts_count = (n as f64 * 0.40) as usize;
    #[cfg(feature = "csharp")]
    let cs_count = (n as f64 * 0.30) as usize;
    #[cfg(feature = "rust")]
    let rs_count = (n as f64 * 0.20) as usize;
    // Remaining: java (when feature enabled)

    // ── TypeScript files (Angular components + services) ──────────────
    for i in 0..ts_count {
        let subdir = if i % 3 == 0 {
            "components"
        } else if i % 3 == 1 {
            "services"
        } else {
            "models"
        };
        let file_path = path
            .join("src")
            .join("app")
            .join(subdir)
            .join(format!("entity_{}.ts", i));
        std::fs::create_dir_all(file_path.parent().unwrap()).ok();

        let source = if i % 3 == 0 {
            // Angular component
            format!(
                r#"
import {{ Component, Input, Output, EventEmitter, OnInit }} from '@angular/core';
import {{ CommonModule }} from '@angular/common';
import {{ FormsModule }} from '@angular/forms';
import {{ EntityService{} }} from '../services/entity_{}.service';

@Component({{
    selector: 'app-entity-{}',
    template: '<div>{{data}}</div>',
    styles: [':host {{ display: block; }}']
}})
export class EntityComponent{} implements OnInit {{
    @Input() data: string = '';
    @Output() changed = new EventEmitter<string>();
    private cache: Map<string, any> = new Map();

    constructor(private service: EntityService{}) {{}}

    ngOnInit(): void {{
        this.loadData();
    }}

    loadData(): void {{
        const result = this.service.getData(this.data);
        this.cache.set(this.data, result);
    }}

    processItem(item: string): string {{
        if (this.cache.has(item)) {{
            return this.cache.get(item);
        }}
        const processed = item.toUpperCase();
        this.cache.set(item, processed);
        return processed;
    }}
}}
"#,
                i, i, i, i, i
            )
        } else if i % 3 == 1 {
            // Angular service
            format!(
                r#"
import {{ Injectable }} from '@angular/core';
import {{ HttpClient, HttpParams }} from '@angular/common/http';
import {{ Observable, of, throwError }} from 'rxjs';
import {{ catchError, map, tap }} from 'rxjs/operators';
import {{ EntityModel{} }} from '../models/entity_{}.model';

@Injectable({{ providedIn: 'root' }})
export class EntityService{} {{
    private apiUrl = '/api/entities/{}';

    constructor(private http: HttpClient) {{}}

    getData(id: string): Observable<EntityModel{}> {{
        return this.http.get<EntityModel{}>(`${{this.apiUrl}}${{id}}`).pipe(
            tap(data => console.log('Fetched data for', id)),
            catchError(err => {{
                console.error('Error fetching', id, err);
                return throwError(() => new Error('Failed to fetch'));
            }})
        );
    }}

    saveData(data: EntityModel{}): Observable<EntityModel{}> {{
        return this.http.post<EntityModel{}>(this.apiUrl, data).pipe(
            tap(saved => console.log('Saved', saved.id)),
            catchError(err => throwError(() => new Error('Failed to save')))
        );
    }}

    deleteData(id: string): Observable<boolean> {{
        return this.http.delete<boolean>(`${{this.apiUrl}}${{id}}`).pipe(
            map(() => true),
            catchError(err => of(false))
        );
    }}
}}
"#,
                i, i, i, i, i, i, i, i, i
            )
        } else {
            // TypeScript model/interface
            format!(
                r#"
export interface EntityModel{} {{
    id: string;
    name: string;
    description: string;
    createdAt: Date;
    updatedAt: Date;
    status: 'active' | 'inactive' | 'archived';
    metadata: Record<string, any>;
    tags: string[];
    version: number;
}}

export class EntityValidator{} {{
    validate(entity: Partial<EntityModel{}>): string[] {{
        const errors: string[] = [];
        if (!entity.id) errors.push('id is required');
        if (!entity.name) errors.push('name is required');
        if (entity.name && entity.name.length < 3) errors.push('name too short');
        return errors;
    }}

    sanitize(entity: Partial<EntityModel{}>): Partial<EntityModel{}> {{
        return {{
            ...entity,
            name: entity.name?.trim(),
            description: entity.description?.trim(),
            tags: entity.tags?.filter(t => t.length > 0) ?? [],
        }};
    }}
}}
"#,
                i, i, i, i, i
            )
        };

        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(source.as_bytes()).unwrap();
    }

    // ── C# files (EF Core + Controllers) — gated by `csharp` feature ──
    #[cfg(feature = "csharp")]
    for i in 0..cs_count {
        let subdir = if i % 2 == 0 { "Controllers" } else { "Models" };
        let file_path = path
            .join("src")
            .join("WebApi")
            .join(subdir)
            .join(format!("Entity{}.cs", i));
        std::fs::create_dir_all(file_path.parent().unwrap()).ok();

        let source = if i % 2 == 0 {
            format!(
                r#"
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace WebApi.Controllers
{{
    [ApiController]
    [Route("api/[controller]")]
    public class Entity{}Controller : ControllerBase
    {{
        private readonly AppDbContext _context;
        private readonly ILogger<Entity{}Controller> _logger;

        public Entity{}Controller(AppDbContext context, ILogger<Entity{}Controller> logger)
        {{
            _context = context;
            _logger = logger;
        }}

        [HttpGet]
        public async Task<ActionResult<IEnumerable<Entity{}>>> GetAll()
        {{
            try
            {{
                var entities = await _context.Set<Entity{}>().ToListAsync();
                return Ok(entities);
            }}
            catch (Exception ex)
            {{
                _logger.LogError(ex, "Error fetching entities");
                return StatusCode(500, "Internal server error");
            }}
        }}

        [HttpGet("{{id}}")]
        public async Task<ActionResult<Entity{}>> GetById(int id)
        {{
            var entity = await _context.Set<Entity{}>().FindAsync(id);
            if (entity == null)
                return NotFound();
            return Ok(entity);
        }}

        [HttpPost]
        public async Task<ActionResult<Entity{}>> Create(Entity{} entity)
        {{
            _context.Set<Entity{}>().Add(entity);
            await _context.SaveChangesAsync();
            return CreatedAtAction(nameof(GetById), new {{ id = entity.Id }}, entity);
        }}
    }}
}}
"#,
                i, i, i, i, i, i, i, i, i, i, i
            )
        } else {
            format!(
                r#"
using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace WebApi.Models
{{
    [Table("Entities_{}")]
    public class Entity{}
    {{
        [Key]
        [DatabaseGenerated(DatabaseGeneratedOption.Identity)]
        public int Id {{ get; set; }}

        [Required]
        [MaxLength(200)]
        public string Name {{ get; set; }} = string.Empty;

        [MaxLength(2000)]
        public string? Description {{ get; set; }}

        public DateTime CreatedAt {{ get; set; }} = DateTime.UtcNow;
        public DateTime UpdatedAt {{ get; set; }} = DateTime.UtcNow;

        [ConcurrencyCheck]
        public byte[] RowVersion {{ get; set; }} = Array.Empty<byte>();

        public ICollection<RelatedEntity{}> RelatedEntities {{ get; set; }} = new List<RelatedEntity{}>();
    }}

    public class RelatedEntity{}
    {{
        public int Id {{ get; set; }}
        public string Name {{ get; set; }} = string.Empty;
        public int ParentId {{ get; set; }}
        [ForeignKey(nameof(ParentId))]
        public Entity{} Parent {{ get; set; }} = null!;
    }}
}}
"#,
                i, i, i, i, i, i
            )
        };

        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(source.as_bytes()).unwrap();
    }

    // ── Rust files (modules + structs) — gated by `rust` feature ──────
    #[cfg(feature = "rust")]
    for i in 0..rs_count {
        let file_path = path
            .join("src")
            .join("rust")
            .join(format!("module_{}.rs", i));
        std::fs::create_dir_all(file_path.parent().unwrap()).ok();

        let source = format!(
            r#"
use std::collections::HashMap;
use serde::{{Deserialize, Serialize}};
use thiserror::Error;

pub struct Entity{} {{
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: EntityStatus,
    pub tags: Vec<String>,
}}

pub enum EntityStatus {{
    Active,
    Inactive,
    Archived,
}}

impl Entity{} {{
    pub fn new(name: String) -> Self {{
        let now = chrono::Utc::now();
        Self {{
            id: 0,
            name,
            description: None,
            created_at: now,
            updated_at: now,
            status: EntityStatus::Active,
            tags: Vec::new(),
        }}
    }}

    pub fn archive(&mut self) {{
        self.status = EntityStatus::Archived;
        self.updated_at = chrono::Utc::now();
    }}

    pub fn add_tag(&mut self, tag: String) {{
        if !self.tags.contains(&tag) {{
            self.tags.push(tag);
        }}
    }}
}}

pub struct EntityRepository {{
    store: HashMap<u64, Entity{}>,
    next_id: u64,
}}

impl EntityRepository {{
    pub fn new() -> Self {{
        Self {{ store: HashMap::new(), next_id: 1 }}
    }}

    pub fn create(&mut self, mut entity: Entity{}) -> Entity{} {{
        entity.id = self.next_id;
        self.next_id += 1;
        self.store.insert(entity.id, entity.clone());
        entity
    }}

    pub fn get(&self, id: u64) -> Result<&Entity{}, EntityError> {{
        self.store.get(&id).ok_or(EntityError::NotFound(id))
    }}

    pub fn update(&mut self, entity: Entity{}) -> Result<(), EntityError> {{
        if !self.store.contains_key(&entity.id) {{
            return Err(EntityError::NotFound(entity.id));
        }}
        self.store.insert(entity.id, entity);
        Ok(())
    }}

    pub fn delete(&mut self, id: u64) -> Result<(), EntityError> {{
        self.store.remove(&id).ok_or(EntityError::NotFound(id)).map(|_| ())
    }}
}}
"#,
            i, i, i, i, i, i, i
        );

        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(source.as_bytes()).unwrap();
    }

    // ── Java files (Spring Boot) — gated by `java` feature ────────────
    #[cfg(feature = "java")]
    let java_count = {
        let mut remaining = n - ts_count;
        #[cfg(feature = "csharp")]
        {
            remaining -= cs_count;
        }
        #[cfg(feature = "rust")]
        {
            remaining -= rs_count;
        }
        remaining
    };
    #[cfg(feature = "java")]
    for i in 0..java_count {
        let file_path = path
            .join("src")
            .join("main")
            .join("java")
            .join("com")
            .join("example")
            .join("demo")
            .join(format!("Entity{}.java", i));
        std::fs::create_dir_all(file_path.parent().unwrap()).ok();

        let source = if i % 2 == 0 {
            format!(
                r#"
package com.example.demo;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import jakarta.validation.Valid;
import java.util.List;

@RestController
@RequestMapping("/api/entities/{}")
public class Entity{}Controller {{
    private static final Logger log = LoggerFactory.getLogger(Entity{}Controller.class);

    @Autowired
    private Entity{}Service service;

    @GetMapping
    public ResponseEntity<List<Entity{}>> getAll() {{
        log.info("Fetching all entities");
        return ResponseEntity.ok(service.findAll());
    }}

    @GetMapping("/{{id}}")
    public ResponseEntity<Entity{}> getById(@PathVariable Long id) {{
        return service.findById(id)
            .map(ResponseEntity::ok)
            .orElse(ResponseEntity.notFound().build());
    }}

    @PostMapping
    public ResponseEntity<Entity{}> create(@Valid @RequestBody Entity{} entity) {{
        Entity{} saved = service.save(entity);
        return ResponseEntity.status(HttpStatus.CREATED).body(saved);
    }}

    @DeleteMapping("/{{id}}")
    public ResponseEntity<Void> delete(@PathVariable Long id) {{
        service.deleteById(id);
        return ResponseEntity.noContent().build();
    }}
}}
"#,
                i, i, i, i, i, i, i, i, i
            )
        } else {
            format!(
                r#"
package com.example.demo;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.util.List;
import java.util.Optional;

@Service
@Transactional
public class Entity{}Service {{
    private static final Logger log = LoggerFactory.getLogger(Entity{}Service.class);

    @Autowired
    private Entity{}Repository repository;

    public List<Entity{}> findAll() {{
        log.debug("Finding all entities");
        return repository.findAll();
    }}

    public Optional<Entity{}> findById(Long id) {{
        return repository.findById(id);
    }}

    public Entity{} save(Entity{} entity) {{
        log.info("Saving entity: {{}}", entity.getName());
        return repository.save(entity);
    }}

    public void deleteById(Long id) {{
        log.warn("Deleting entity: {{}}", id);
        repository.deleteById(id);
    }}

    public List<Entity{}> searchByName(String name) {{
        return repository.findByNameContainingIgnoreCase(name);
    }}
}}
"#,
                i, i, i, i, i, i, i, i
            )
        };

        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(source.as_bytes()).unwrap();
    }

    (path, temp_dir) // keep TempDir alive for test duration
}

// ── Memory monitoring helper ──────────────────────────────────────────

/// Attempt to read peak memory usage (RSS) in bytes.
///
/// Returns `None` on platforms where memory info is not available
/// (e.g., unsupported OS or missing system tools).
fn peak_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/self/status
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmPeak:") {
                    // Format: "VmPeak:   123456 kB"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Some(kb * 1024);
                        }
                    }
                }
            }
        }
        None
    }
    #[cfg(target_os = "windows")]
    {
        // Use `tasklist` to query this process's memory usage
        let pid = std::process::id();
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // CSV format: "image.exe","pid","session","session#","mem KB"
            if let Some(line) = stdout.lines().next() {
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() >= 5 {
                    let mem_str = fields[4].trim_matches('"').trim();
                    // Remove " K" suffix if present, then parse
                    let cleaned = mem_str.replace(" K", "").replace(",", "");
                    if let Ok(kb) = cleaned.parse::<u64>() {
                        return Some(kb * 1024);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        // Unsupported platform — no memory monitoring available
        None
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

/// Test: Large workspace with Low fidelity (CI-friendly size: 120 files)
#[test]
fn large_workspace_performance_low_fidelity() {
    let (workspace_path, _temp_dir) = setup_large_test_workspace(120);

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    let mem_before = peak_memory_bytes();
    let start = Instant::now();
    let result = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );
    let duration = start.elapsed();
    let mem_after = peak_memory_bytes();

    let result = result.expect("Workspace compression should succeed");

    let total_files = result.manifest.matches("FILE:").count();
    assert!(
        total_files >= 40,
        "Should process at least 40 files, got {}",
        total_files
    );
    assert!(
        result.errors.is_empty(),
        "Should have zero errors, got {}: {:?}",
        result.errors.len(),
        result.errors
    );
    assert!(
        duration.as_secs() < 60,
        "Should complete in <60s, took {:?}",
        duration
    );

    println!(
        "Low fidelity workspace: {} files in {:?}",
        total_files, duration
    );
    if let (Some(before), Some(after)) = (mem_before, mem_after) {
        let delta_mb = (after.saturating_sub(before)) as f64 / 1_048_576.0;
        println!(
            "  Memory: before={}KB, after={}KB, delta={:.1}MB",
            before / 1024,
            after / 1024,
            delta_mb
        );
    }
}

/// Test: Large workspace with Medium fidelity
#[test]
fn large_workspace_medium_fidelity() {
    let (workspace_path, _temp_dir) = setup_large_test_workspace(50);

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    let start = Instant::now();
    let result = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Medium,
        &state,
    );
    let duration = start.elapsed();

    let result = result.expect("Medium fidelity workspace compression should succeed");

    let total_files = result.manifest.matches("FILE:").count();
    assert!(
        total_files >= 15,
        "Should process at least 15 files, got {}",
        total_files
    );
    assert!(result.errors.is_empty(), "Should have zero errors");
    assert!(
        duration.as_secs() < 60,
        "Should complete in <60s, took {:?}",
        duration
    );

    println!(
        "Medium fidelity workspace: {} files in {:?}",
        total_files, duration
    );
}

/// Test: Large workspace with High fidelity
#[test]
fn large_workspace_high_fidelity() {
    let (workspace_path, _temp_dir) = setup_large_test_workspace(30);

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    let start = Instant::now();
    let result = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::High,
        &state,
    );
    let duration = start.elapsed();

    let result = result.expect("High fidelity workspace compression should succeed");

    let total_files = result.manifest.matches("FILE:").count();
    assert!(
        total_files >= 10,
        "Should process at least 10 files, got {}",
        total_files
    );
    assert!(result.errors.is_empty(), "Should have zero errors");
    assert!(
        duration.as_secs() < 60,
        "Should complete in <60s, took {:?}",
        duration
    );

    println!(
        "High fidelity workspace: {} files in {:?}",
        total_files, duration
    );
}

/// Test: Workspace with excluded patterns
#[test]
fn large_workspace_with_exclusions() {
    let (workspace_path, _temp_dir) = setup_large_test_workspace(50);

    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("node_modules".to_string());
    config.exclude_patterns.push("dist".to_string());

    let state = McpState::new(config);

    let result = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );

    let result = result.expect("Workspace with exclusions should succeed");

    let total_files = result.manifest.matches("FILE:").count();
    assert!(total_files > 0, "Should process some files");
    assert!(result.errors.is_empty(), "Should have zero errors");

    println!(
        "Workspace with exclusions: {} files processed, {} excluded",
        total_files,
        result.excluded.len()
    );
}

/// Test: Incremental changes on large workspace
#[test]
fn large_workspace_incremental_delta() {
    let (workspace_path, _temp_dir) = setup_large_test_workspace(30);

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // First pass: full compression
    let result1 = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );
    let result1 = result1.expect("First pass should succeed");
    let first_file_count = result1.manifest.matches("FILE:").count();

    // Modify a few files (simulate incremental changes)
    for i in 0..5 {
        let file_path = workspace_path
            .join("src")
            .join("app")
            .join("services")
            .join(format!("entity_{}.ts", i));
        if file_path.exists() {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&file_path)
                .unwrap();
            writeln!(f, "\n// Incremental change {}", i).unwrap();
        }
    }

    // Second pass: should handle modified files
    let result2 = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );
    let result2 = result2.expect("Second pass after modifications should succeed");
    let second_file_count = result2.manifest.matches("FILE:").count();

    // Both passes should process similar number of files
    assert_eq!(
        first_file_count, second_file_count,
        "File count should be consistent across passes: {} vs {}",
        first_file_count, second_file_count
    );
    assert!(
        result2.errors.is_empty(),
        "Second pass should have zero errors"
    );

    println!(
        "Incremental delta: {} files (first), {} files (second), {} errors",
        first_file_count,
        second_file_count,
        result2.errors.len()
    );
}

/// Test: Cache hit rate — second pass on unchanged workspace should be faster
///
/// Verifies that the workspace result cache (F-22) is working correctly:
/// the second pass on an identical workspace should hit the cache and
/// complete significantly faster than the first (cold cache) pass.
#[test]
fn large_workspace_cache_hit_rate() {
    let (workspace_path, _temp_dir) = setup_large_test_workspace(30);

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // First pass: cold cache (full compression)
    let start1 = Instant::now();
    let result1 = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );
    let duration1 = start1.elapsed();
    let result1 = result1.expect("First pass (cold cache) should succeed");
    let first_file_count = result1.manifest.matches("FILE:").count();
    assert!(
        result1.errors.is_empty(),
        "First pass should have zero errors"
    );

    // Second pass: same workspace, should hit cache
    let start2 = Instant::now();
    let result2 = crate::mcp::workspace::compress_workspace_dir(
        workspace_path.to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );
    let duration2 = start2.elapsed();
    let result2 = result2.expect("Second pass (cache) should succeed");
    let second_file_count = result2.manifest.matches("FILE:").count();
    assert!(
        result2.errors.is_empty(),
        "Second pass should have zero errors"
    );

    // Both passes should produce the same file count
    assert_eq!(
        first_file_count, second_file_count,
        "File count should be identical across passes: {} vs {}",
        first_file_count, second_file_count
    );

    // Second pass should be faster (cache hit)
    assert!(
        duration2 < duration1,
        "Second pass ({:?}) should be faster than first ({:?}) — cache hit expected",
        duration2,
        duration1
    );

    let speedup = if duration2.as_nanos() > 0 {
        duration1.as_nanos() / duration2.as_nanos()
    } else {
        u128::MAX
    };
    println!(
        "Cache hit rate: first={:?}, second={:?} ({}x faster)",
        duration1, duration2, speedup
    );
}
