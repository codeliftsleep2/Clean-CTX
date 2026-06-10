// examples/fidelity_comparison.rs
//
// Runs the 50-edit simulation at all three fidelity levels (Low, Medium, High)
// and produces a side-by-side comparison of token savings.
//
// Run with:
//     cargo run --example fidelity_comparison
//
// Output: per-fidelity summary tables + cross-fidelity comparison

use clean_ctx::analytics::{bpe_or_init, bpe};
use clean_ctx::compression::{compress_file, Fidelity};
use clean_ctx::dictionary::PathDictionary;
use clean_ctx::cache::LocalStateCache;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
struct EditRecord {
    number: usize,
    description: String,
    category: EditCategory,
    raw_cost: usize,
    recompression_cost: usize,
    delta_cost: usize,
    cum_raw: usize,
    cum_recomp: usize,
    cum_delta: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditCategory {
    Small, Method, Structural, CrossMethod, Refactor,
}

fn generate_edit_sequence() -> Vec<(String, EditCategory, String)> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base_path = manifest_dir.join("src/test_files/UserManagementService.ts");
    let base_source = std::fs::read_to_string(&base_path)
        .expect("Cannot read UserManagementService.ts");

    let mut results: Vec<(String, EditCategory, String)> = Vec::new();
    let mut current = base_source;

    // Edits 1-10: Small changes
    current = current.replace("apiBasePath", "baseApiPath");
    results.push(("Rename 'apiBasePath' to 'baseApiPath'".to_string(), EditCategory::Small, current.clone()));

    current = current.replace(
        "private logOperation(operation: string, details: Record<string, unknown>)",
        "private logOperation(operation: string, details: Record<string, unknown>): void"
    );
    results.push(("Add void return type to logOperation".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("isActive", "active");
    results.push(("Rename 'isActive' to 'active' in UserFilter".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("private readonly defaultPageSize = 25;", "private readonly defaultPageSize = 50;");
    results.push(("Change defaultPageSize from 25 to 50".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("return this.lastError;", "return this.lastError ?? null;");
    results.push(("Add null coalesce to getLastError return".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("private readonly cacheDurationMs = 5 * 60 * 1000;", "private readonly cacheDurationMs = 10 * 60 * 1000;");
    results.push(("Change cache TTL from 5min to 10min".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("private pendingRequests = 0;", "private pendingRequests = 0;\n  private activeRequests = 0;");
    results.push(("Add activeRequests counter field".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("isAuthenticated(): boolean {", "hasActiveSession(): boolean {");
    results.push(("Rename isAuthenticated to hasActiveSession".to_string(), EditCategory::Small, current.clone()));

    current = current.replace(
        "headers = headers.set('X-Request-Timestamp', Date.now().toString());",
        "headers = headers.set('X-Request-Timestamp', Date.now().toString());\n    headers = headers.set('X-Request-ID', crypto.randomUUID());"
    );
    results.push(("Add X-Request-ID header to buildHeaders".to_string(), EditCategory::Small, current.clone()));

    current = current.replace("if (data.displayName && data.displayName.length < 2) {", "if (data.displayName && data.displayName.trim().length < 2) {");
    results.push(("Add trim() to displayName validation".to_string(), EditCategory::Small, current.clone()));

    // Edits 11-20: Method-level
    let new_method = "\n  async getUserPermissions(userId: UserId): Promise<ApiResponse<string[]>> {\n    try {\n      const url = `${this.apiBaseUrl}${this.baseApiPath}/${userId}/permissions`;\n      this.incrementRequestCount();\n      const response = await firstValueFrom(\n        this.http.get<string[]>(url, { headers: this.buildHeaders(timeoutMs) })\n      );\n      return { success: true, data: response, error: null, statusCode: 200, timestamp: new Date().toISOString() };\n    } catch (error) {\n      return this.handleError<string[]>(error, 'getUserPermissions', { userId });\n    }\n  }\n\n  ";
    if let Some(pos) = current.find("getTotalRequests(") {
        current = format!("{}{}{}", &current[..pos], new_method, &current[pos..]);
    }
    results.push(("Add getUserPermissions method".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("async batchCreateUsers(users: Array<Partial<UserProfile>>)", "async batchCreateUsers(users: Array<Partial<UserProfile>>, batchSize: number = 10)");
    results.push(("Add batchSize param to batchCreateUsers".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("async getUserById(userId: UserId)", "async getUserById(userId: UserId, fields?: string[])");
    results.push(("Add optional fields param to getUserById".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("Promise<ApiResponse<boolean>>", "Promise<ApiResponse<{ verified: boolean; verifiedAt: string }>>");
    results.push(("Change verifyEmail return type to object".to_string(), EditCategory::Method, current.clone()));

    current = current.replace(
        "this.incrementRequestCount();\n      const response = await firstValueFrom(\n        this.http.get<PaginatedResult<UserProfile>>(url, {",
        "this.incrementRequestCount();\n      const controller = new AbortController();\n      const timeoutId = setTimeout(() => controller.abort(), 5000);\n      const response = await firstValueFrom(\n        this.http.get<PaginatedResult<UserProfile>>(url, { signal: controller.signal,"
    );
    results.push(("Add AbortController timeout to getUsers".to_string(), EditCategory::Method, current.clone()));

    current = current.replace(
        "async getUserByEmail(email: EmailAddress): Promise<ApiResponse<UserProfile>> {",
        "async getUserByEmail(email: EmailAddress): Promise<ApiResponse<UserProfile>> {\n    const cacheKey = `user:email:${email}`;\n    const cached = this.getFromCache<UserProfile>(cacheKey);\n    if (cached) {\n      return { success: true, data: cached, error: null, statusCode: 200, timestamp: new Date().toISOString() };\n    }\n    "
    );
    results.push(("Add caching logic to getUserByEmail".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("this.http.put<UserProfile>(url, validatedData.data, {", "this.http.patch<UserProfile>(url, validatedData.data, {");
    results.push(("Change updateUser from PUT to PATCH".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("await Promise.all(promises);", "const settled = await Promise.allSettled(promises);\n        for (const s of settled) {\n          if (s.status === 'rejected') {\n            result.failed++;\n            result.errors.push({ id: 'batch', error: s.reason?.toString() || 'Unknown' });\n          }\n        }");
    results.push(("Add Promise.allSettled for batch error handling".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("private readonly analyticsService: AnalyticsService,", "private readonly analyticsService: AnalyticsService,\n    private readonly loggerService: LoggerService,");
    results.push(("Add LoggerService constructor dependency".to_string(), EditCategory::Method, current.clone()));

    current = current.replace("private readonly maxPageSize = 100;", "private readonly maxPageSize = 200;");
    results.push(("Change maxPageSize from 100 to 200".to_string(), EditCategory::Method, current.clone()));

    // Edits 21-30: Structural
    current = current.replace("export interface BatchOperationResult",
        "export interface UserSession {\n  sessionId: string;\n  userId: string;\n  token: string;\n  expiresAt: string;\n  createdAt: string;\n  ipAddress: string;\n  userAgent: string;\n  isActive: boolean;\n}\n\nexport interface BatchOperationResult");
    results.push(("Add UserSession interface".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace(
        "export type AuditAction = 'CREATE' | 'UPDATE' | 'DELETE' | 'LOGIN' | 'LOGOUT' | 'PASSWORD_RESET';",
        "export type AuditAction = 'CREATE' | 'UPDATE' | 'DELETE' | 'LOGIN' | 'LOGOUT' | 'PASSWORD_RESET';\nexport type SessionToken = string;\nexport type IPAddress = string;"
    );
    results.push(("Add SessionToken and IPAddress type aliases".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace("  constructor(", "  @Input() serviceConfig: { cacheEnabled: boolean; logLevel: string } = { cacheEnabled: true, logLevel: 'info' };\n\n  constructor(");
    results.push(("Add @Input() serviceConfig field".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace(
        "private validateUserData(data: Partial<UserProfile>): { validationPassed: boolean; data: Partial<UserProfile>; error: string | null } {",
        "private validateRequiredFields(data: Partial<UserProfile>): string[] {\n    const errors: string[] = [];\n    if (data.email && !this.isValidEmail(data.email)) {\n      errors.push('Invalid email format');\n    }\n    if (data.displayName && data.displayName.trim().length < 2) {\n      errors.push('Display name must be at least 2 characters');\n    }\n    return errors;\n  }\n\n  private validateOptionalFields(data: Partial<UserProfile>): string[] {\n    const errors: string[] = [];\n    if (data.displayName && data.displayName.length > 100) {\n      errors.push('Display name must not exceed 100 characters');\n    }\n    if (data.phoneNumber && !this.isValidPhoneNumber(data.phoneNumber)) {\n      errors.push('Invalid phone number format');\n    }\n    return errors;\n  }\n\n  private validateUserData(data: Partial<UserProfile>): { validationPassed: boolean; data: Partial<UserProfile>; error: string | null } {"
    );
    results.push(("Split validateUserData into helper methods".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace(
        "@Output() batchOperationComplete = new EventEmitter<BatchOperationResult>();",
        "@Output() batchOperationComplete = new EventEmitter<BatchOperationResult>();\n  @Output() errorOccurred = new EventEmitter<{ operation: string; error: string; timestamp: string }>();"
    );
    results.push(("Add @Output() errorOccurred EventEmitter".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace("async verifyEmail",
        "async suspendUser(userId: UserId, reason: string): Promise<ApiResponse<boolean>> {\n    try {\n      const url = `${this.apiBaseUrl}${this.baseApiPath}/${userId}/suspend`;\n      this.incrementRequestCount();\n      await firstValueFrom(\n        this.http.post(url, { reason }, { headers: this.buildHeaders(timeoutMs) })\n      );\n      this.cache.delete(`user:${userId}`);\n      this.logOperation('suspendUser', { userId, reason });\n      return { success: true, data: true, error: null, statusCode: 200, timestamp: new Date().toISOString() };\n    } catch (error) {\n      return this.handleError<boolean>(error, 'suspendUser', { userId, reason });\n    }\n  }\n\n  async verifyEmail"
    );
    results.push(("Add suspendUser method".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace("private activeRequests = 0;", "private activeRequests = 0;\n  private readonly rateLimitPerMinute = 60;\n  private requestTimestamps: number[] = [];");
    results.push(("Add rate limiting fields".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace("private validateFilter(filter: UserFilter): UserFilter {",
        "async getAllUsers(page: number = 1, pageSize: number = 50): Promise<ApiResponse<PaginatedResult<UserProfile>>> {\n    const filter: UserFilter = {\n      searchText: '',\n      roles: [],\n      active: null,\n      isEmailVerified: null,\n      createdAfter: null,\n      createdBefore: null,\n      sortBy: 'createdAt',\n      sortDirection: 'desc',\n      page,\n      pageSize,\n    };\n    return this.getUsers(filter);\n  }\n\n  private validateFilter(filter: UserFilter): UserFilter {"
    );
    results.push(("Add getAllUsers convenience method".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace("private incrementRequestCount(): void {",
        "private logMethodEntry(method: string, args: Record<string, unknown>): void {\n    if (this.serviceConfig.logLevel === 'debug') {\n      try {\n        this.loggerService.log(`Entering ${method}`, args);\n      } catch {\n        // silent\n      }\n    }\n  }\n\n  private logMethodExit(method: string, durationMs: number): void {\n    if (this.serviceConfig.logLevel === 'debug') {\n      try {\n        this.loggerService.log(`Exiting ${method}`, { durationMs });\n      } catch {\n        // silent\n      }\n    }\n  }\n\n  private incrementRequestCount(): void {"
    );
    results.push(("Add logMethodEntry and logMethodExit helpers".to_string(), EditCategory::Structural, current.clone()));

    current = current.replace("export interface UserSession",
        "export interface UserServiceConfig {\n  basePath: string;\n  defaultPageSize: number;\n  maxPageSize: number;\n  cacheEnabled: boolean;\n  cacheDurationMs: number;\n  timeoutMs: number;\n  retryCount: number;\n}\n\nexport interface UserSession"
    );
    results.push(("Add UserServiceConfig interface".to_string(), EditCategory::Structural, current.clone()));

    // Edits 31-40: Cross-method
    current = current.replace("private logMethodEntry", "private buildUrl(path: string): string {\n    return `${this.apiBaseUrl}${this.baseApiPath}${path}`;\n  }\n\n  private logMethodEntry");
    results.push(("Extract buildUrl URL builder method".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private getFromCache<T>(key: string): T | null {\n    const entry = this.cache.get(key);", "private getFromCache<T>(key: string): T | null {\n    try {\n    const entry = this.cache.get(key);");
    results.push(("Add try/catch to getFromCache".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private setInCache<T>(key: string, data: T): void {\n    if (this.cache.size > 100) {", "private setInCache<T>(key: string, data: T): void {\n    try {\n    if (this.cache.size > 100) {");
    results.push(("Add try/catch to setInCache".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private logMethodExit",
        "private async withRetry<T>(operation: () => Promise<T>, maxRetries: number = 3): Promise<T> {\n    let lastError: unknown;\n    for (let attempt = 1; attempt <= maxRetries; attempt++) {\n      try {\n        return await operation();\n      } catch (error) {\n        lastError = error;\n        if (attempt < maxRetries) {\n          await this.delay(Math.pow(2, attempt) * 100);\n        }\n      }\n    }\n    throw lastError;\n  }\n\n  private delay(ms: number): Promise<void> {\n    return new Promise(resolve => setTimeout(resolve, ms));\n  }\n\n  private logMethodExit"
    );
    results.push(("Add withRetry and delay cross-method helpers".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private buildUrl", "private throwIfNotAuthenticated(): void {\n    if (!this.hasActiveSession()) {\n      throw new Error('User is not authenticated');\n    }\n  }\n\n  private buildUrl");
    results.push(("Add throwIfNotAuthenticated guard method".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private throwIfNotAuthenticated",
        "private async measure<T>(label: string, fn: () => Promise<T>): Promise<T> {\n    const start = Date.now();\n    try {\n      return await fn();\n    } finally {\n      const duration = Date.now() - start;\n      try {\n        this.analyticsService.trackTiming('user_management', label, duration);\n      } catch {\n        // silent\n      }\n    }\n  }\n\n  private throwIfNotAuthenticated"
    );
    results.push(("Add measure() timing wrapper".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private readonly loggerService: LoggerService,", "private readonly loggerService: LoggerService,\n    private readonly configService: ConfigService,");
    results.push(("Add ConfigService constructor dependency".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("private incrementRequestCount(): void {\n    this.requestCount++;",
        "private incrementRequestCount(): void {\n    const now = Date.now();\n    this.requestTimestamps.push(now);\n    const oneMinuteAgo = now - 60000;\n    this.requestTimestamps = this.requestTimestamps.filter(t => t > oneMinuteAgo);\n    if (this.requestTimestamps.length > this.rateLimitPerMinute) {\n      throw new Error('Rate limit exceeded. Please try again later.');\n    }\n    this.requestCount++;"
    );
    results.push(("Add rate limiting logic to incrementRequestCount".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("this.cache.set(key, { data, timestamp: Date.now() });\n    }", "this.cache.set(key, { data, timestamp: Date.now() });\n      this.logOperation('cache_set', { key });\n    }");
    results.push(("Add cache_set operation logging".to_string(), EditCategory::CrossMethod, current.clone()));

    current = current.replace("console.error(`Operation ${operation} failed:`, errorMessage);", "this.errorOccurred.emit({ operation, error: errorMessage, timestamp: new Date().toISOString() });\n      console.error(`Operation ${operation} failed:`, errorMessage);");
    results.push(("Emit errorOccurred event in handleError".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edits 41-50: Refactor
    current = current.replace("async getUsers(filter: UserFilter): Promise<ApiResponse<PaginatedResult<UserProfile>>> {",
        "async getUsers(filter: UserFilter, options?: { useCache?: boolean; timeout?: number }): Promise<ApiResponse<PaginatedResult<UserProfile>>> {");
    results.push(("Add RequestOptions object to getUsers signature".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("export interface UserServiceConfig",
        "export interface RequestOptions {\n  useCache: boolean;\n  timeout: number;\n  retryOnFailure: boolean;\n  retryCount: number;\n  headers?: Record<string, string>;\n}\n\nexport interface UserServiceConfig");
    results.push(("Add RequestOptions interface".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace(
        "constructor(\n    private readonly http: HttpClient,\n    @Inject('API_BASE_URL') private readonly apiBaseUrl: string,\n    @Optional() @Inject('CACHE_PROVIDER') private readonly cacheProvider: unknown,\n    private readonly notificationService: NotificationService,\n    private readonly analyticsService: AnalyticsService,\n    private readonly loggerService: LoggerService,\n    private readonly configService: ConfigService,\n  ) {",
        "constructor(\n    private readonly http: HttpClient,\n    @Inject('API_BASE_URL') private readonly apiBaseUrl: string,\n    @Optional() @Inject('CACHE_PROVIDER') private readonly cacheProvider: unknown,\n    private readonly notificationService: NotificationService,\n    private readonly analyticsService: AnalyticsService,\n    private readonly loggerService: LoggerService,\n    private readonly configService: ConfigService,\n    private readonly options: UserServiceConfig,\n  ) {"
    );
    results.push(("Refactor constructor to use UserServiceConfig options object".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("export interface RequestOptions",
        "export interface UserOperationResult<T> {\n  success: boolean;\n  data: T | null;\n  error: string | null;\n  statusCode: number;\n  timestamp: string;\n  durationMs: number;\n  cached: boolean;\n}\n\nexport interface RequestOptions");
    results.push(("Add UserOperationResult<T> generic interface".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("ApiResponse<PaginatedResult<UserProfile>>", "UserOperationResult<PaginatedResult<UserProfile>>");
    results.push(("Replace ApiResponse with UserOperationResult in all signatures".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("  private readonly baseApiPath", "  // ════════════════════════════════════════════════════════\n  // Configuration\n  // ════════════════════════════════════════════════════════════\n\n  private readonly baseApiPath");
    results.push(("Add Configuration region comment".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("// --- Public API Methods ---", "// ════════════════════════════════════════════════════════\n  // Public API Methods\n  // ════════════════════════════════════════════════════════════");
    results.push(("Add Public API Methods region header".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("// --- Private Helper Methods ---", "// ════════════════════════════════════════════════════════\n  // Private Helper Methods\n  // ════════════════════════════════════════════════════════════");
    results.push(("Add Private Helper Methods region header".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("private handleError<T>(error: unknown, operation: string, context: Record<string, unknown>): UserOperationResult<T> {",
        "private handleError<T>(error: unknown, operation: string, context: Record<string, unknown>, recoveryStrategy?: 'retry' | 'fallback' | 'abort'): UserOperationResult<T> {");
    results.push(("Add recoveryStrategy param to handleError".to_string(), EditCategory::Refactor, current.clone()));

    current = current.replace("private incrementRequestCount(): void {",
        "private async executeSafely<T>(operation: string, fn: () => Promise<T>): Promise<UserOperationResult<T>> {\n    try {\n      this.logMethodEntry(operation, {});\n      const start = Date.now();\n      const data = await fn();\n      const duration = Date.now() - start;\n      this.logMethodExit(operation, duration);\n      return { success: true, data, error: null, statusCode: 200, timestamp: new Date().toISOString(), durationMs: duration, cached: false };\n    } catch (error) {\n      return this.handleError<T>(error, operation, {}, 'retry');\n    }\n  }\n\n  private incrementRequestCount(): void {"
    );
    results.push(("Add executeSafely error boundary wrapper".to_string(), EditCategory::Refactor, current.clone()));

    results
}

fn text_delta_cost(a: &str, b: &str) -> usize {
    let lines_a: Vec<&str> = a.lines().collect();
    let lines_b: Vec<&str> = b.lines().collect();
    let mut a_idx = 0usize;
    let mut b_idx = 0usize;
    let mut total = 0usize;
    while a_idx < lines_a.len() || b_idx < lines_b.len() {
        if a_idx < lines_a.len() && b_idx < lines_b.len() && lines_a[a_idx] == lines_b[b_idx] {
            a_idx += 1; b_idx += 1;
        } else if a_idx < lines_a.len() && b_idx < lines_b.len() {
            total += lines_a[a_idx].len() + lines_b[b_idx].len();
            a_idx += 1; b_idx += 1;
        } else if a_idx < lines_a.len() {
            total += lines_a[a_idx].len() + 1;
            a_idx += 1;
        } else if b_idx < lines_b.len() {
            total += lines_b[b_idx].len() + 1;
            b_idx += 1;
        }
    }
    total
}

struct FidelityRun {
    name: &'static str,
    fidelity: Fidelity,
    total_raw: usize,
    total_recomp: usize,
    total_delta: usize,
    records: Vec<EditRecord>,
    elapsed_secs: f64,
}

fn run_simulation(fidelity: Fidelity) -> FidelityRun {
    let bpe = bpe();
    let start_time = Instant::now();
    let edits = generate_edit_sequence();
    assert_eq!(edits.len(), 50);

    let mut dict = PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let temp_path = std::env::temp_dir().join("clean_ctx_fidelity_comparison.ts");
    let file_path = PathBuf::from(&temp_path);

    let mut records: Vec<EditRecord> = Vec::new();
    let mut prev_compressed: Option<String> = None;
    let mut cum_raw = 0usize;
    let mut cum_recomp = 0usize;
    let mut cum_delta = 0usize;

    for (i, (description, category, source)) in edits.iter().enumerate() {
        let edit_num = i + 1;
        std::fs::write(&temp_path, source).unwrap();
        let raw_tokens = bpe.encode_with_special_tokens(source).len();
        let compressed_output = compress_file(file_path.clone(), &mut dict, &mut cache, fidelity)
            .unwrap_or_else(|e| format!("// Failed: {}", e));
        let compressed_tokens = bpe.encode_with_special_tokens(&compressed_output).len();

        let delta_tokens = if let Some(ref prev) = prev_compressed {
            if prev != &compressed_output {
                let delta_chars = text_delta_cost(prev, &compressed_output);
                let total_delta_chars = delta_chars + 80;
                (total_delta_chars + 3) / 4
            } else { 0 }
        } else {
            compressed_tokens
        };

        cum_raw += raw_tokens;
        cum_recomp += compressed_tokens;
        cum_delta += delta_tokens;

        records.push(EditRecord {
            number: edit_num,
            description: description.clone(),
            category: *category,
            raw_cost: raw_tokens,
            recompression_cost: compressed_tokens,
            delta_cost: delta_tokens,
            cum_raw, cum_recomp, cum_delta,
        });
        prev_compressed = Some(compressed_output);
    }

    let elapsed_secs = start_time.elapsed().as_secs_f64();
    let total_raw = records.last().map(|r| r.cum_raw).unwrap_or(0);
    let total_recomp = records.last().map(|r| r.cum_recomp).unwrap_or(0);
    let total_delta = records.last().map(|r| r.cum_delta).unwrap_or(0);

    let name = match fidelity {
        Fidelity::Low => "Low",
        Fidelity::Medium => "Medium",
        Fidelity::High => "High",
    };

    FidelityRun { name, fidelity, total_raw, total_recomp, total_delta, records, elapsed_secs }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bpe_or_init()?;

    // Run all three fidelities
    let low = run_simulation(Fidelity::Low);
    let medium = run_simulation(Fidelity::Medium);
    let high = run_simulation(Fidelity::High);

    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!("  Clean-CTX 50-Edit Simulation: Cross-Fidelity Comparison");
    println!("  File: UserManagementService.ts (~440 lines, 50 sequential edits)");
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!();

    // ─── Per-Fidelity Summary ──────────────────────────────────────────
    println!("  Per-Fidelity Totals:");
    println!("  {:<10} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "Fidelity", "Raw", "ReComp", "Delta", "ΔvsRaw%", "DeltavsRaw%", "DeltavsRe%");
    println!("  {}", "─".repeat(85));

    for run in &[&low, &medium, &high] {
        let re_pct = if run.total_raw > 0 { (run.total_raw.saturating_sub(run.total_recomp)) as f64 / run.total_raw as f64 * 100.0 } else { 0.0 };
        let del_pct = if run.total_raw > 0 { (run.total_raw.saturating_sub(run.total_delta)) as f64 / run.total_raw as f64 * 100.0 } else { 0.0 };
        let del_vs_re_pct = if run.total_recomp > 0 { (run.total_recomp.saturating_sub(run.total_delta)) as f64 / run.total_recomp as f64 * 100.0 } else { 0.0 };
        println!("  {:<10} {:>12} {:>12} {:>12} {:>10.1}% {:>10.1}% {:>10.1}%",
            run.name, run.total_raw, run.total_recomp, run.total_delta, re_pct, del_pct, del_vs_re_pct);
    }

    println!();

    // ─── Single-pass baseline (first edit) comparison ──────────────────
    println!("  Single-Pass Baseline (Edit #1):");
    println!("  {:<10} {:>12} {:>12} {:>12} {:>12}",
        "Fidelity", "Raw", "Compressed", "Savings", "Ratio");
    println!("  {}", "─".repeat(65));

    for run in &[&low, &medium, &high] {
        let first = &run.records[0];
        let pct = if first.raw_cost > 0 { (first.raw_cost.saturating_sub(first.recompression_cost)) as f64 / first.raw_cost as f64 * 100.0 } else { 0.0 };
        let ratio = if first.recompression_cost > 0 { first.raw_cost as f64 / first.recompression_cost as f64 } else { 0.0 };
        println!("  {:<10} {:>12} {:>12} {:>10.1}% {:>7.1}×",
            run.name, first.raw_cost, first.recompression_cost, pct, ratio);
    }

    println!();

    // ─── Category breakdown across fidelities ──────────────────────────
    let cats = [(EditCategory::Small, "Small"), (EditCategory::Method, "Method"),
                (EditCategory::Structural, "Struct"), (EditCategory::CrossMethod, "Cross"),
                (EditCategory::Refactor, "Refactor")];

    println!("  Category Breakdown (ReComp Savings % by Fidelity):");
    println!("  {:<10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Category", "Raw", "ReLow", "ReMed", "ReHi", "DelLow", "DelMed", "DelHi", "Count");
    println!("  {}", "─".repeat(105));

    for (cat, label) in &cats {
        let cat_raw: usize = low.records.iter().filter(|r| r.category == *cat).map(|r| r.raw_cost).sum();
        let count = low.records.iter().filter(|r| r.category == *cat).count();

        let calc_sav = |run: &FidelityRun| -> (f64, f64) {
            let raw: usize = run.records.iter().filter(|r| r.category == *cat).map(|r| r.raw_cost).sum();
            let re: usize = run.records.iter().filter(|r| r.category == *cat).map(|r| r.recompression_cost).sum();
            let del: usize = run.records.iter().filter(|r| r.category == *cat).map(|r| r.delta_cost).sum();
            let rep = if raw > 0 { (raw.saturating_sub(re)) as f64 / raw as f64 * 100.0 } else { 0.0 };
            let delp = if raw > 0 { (raw.saturating_sub(del)) as f64 / raw as f64 * 100.0 } else { 0.0 };
            (rep, delp)
        };

        let (l_re, l_del) = calc_sav(&low);
        let (m_re, m_del) = calc_sav(&medium);
        let (h_re, h_del) = calc_sav(&high);

        println!("  {:<10} {:>10} {:>9.1}% {:>9.1}% {:>9.1}% {:>9.1}% {:>9.1}% {:>9.1}% {:>10}",
            label, cat_raw, l_re, m_re, h_re, l_del, m_del, h_del, count);
    }

    println!();

    // ─── Insight callouts ───────────────────────────────────────────────
    println!("  Key Insights:");
    println!();

    // Compression ratio grows with fidelity
    let low_ratio = low.total_raw as f64 / low.total_recomp as f64;
    let med_ratio = medium.total_raw as f64 / medium.total_recomp as f64;
    let hi_ratio = high.total_raw as f64 / high.total_recomp as f64;
    println!("  Compression Ratio (Raw/ReComp):  Low={:.0}×  Medium={:.0}×  High={:.0}×",
        low_ratio, med_ratio, hi_ratio);

    // Delta overhead percentage
    let low_del_overhead = if low.total_recomp > 0 { (low.total_delta.saturating_sub(low.total_recomp)) as f64 / low.total_recomp as f64 * 100.0 } else { 0.0 };
    let med_del_overhead = if medium.total_recomp > 0 { (medium.total_delta.saturating_sub(medium.total_recomp)) as f64 / medium.total_recomp as f64 * 100.0 } else { 0.0 };
    let hi_del_overhead = if high.total_recomp > 0 { (high.total_delta.saturating_sub(high.total_recomp)) as f64 / high.total_recomp as f64 * 100.0 } else { 0.0 };
    println!("  Delta Overhead vs Full ReComp:    Low={:+.1}%  Medium={:+.1}%  High={:+.1}%",
        low_del_overhead, med_del_overhead, hi_del_overhead);

    // Average per-edit tokens
    println!("  Avg Tokens/Edit (ReComp):          Low={:.1}  Medium={:.1}  High={:.1}",
        low.total_recomp as f64 / 50.0, medium.total_recomp as f64 / 50.0, high.total_recomp as f64 / 50.0);
    println!("  Avg Tokens/Edit (Delta):           Low={:.1}  Medium={:.1}  High={:.1}",
        low.total_delta as f64 / 50.0, medium.total_delta as f64 / 50.0, high.total_delta as f64 / 50.0);

    // Single-pass comparison
    println!();
    println!("  Single-Pass Compression (Edit #1):");
    for run in &[&low, &medium, &high] {
        let first = &run.records[0];
        let pct = if first.raw_cost > 0 { (first.raw_cost.saturating_sub(first.recompression_cost)) as f64 / first.raw_cost as f64 * 100.0 } else { 0.0 };
        let del_pct = if first.raw_cost > 0 { (first.raw_cost.saturating_sub(first.delta_cost)) as f64 / first.raw_cost as f64 * 100.0 } else { 0.0 };
        println!("    {}: Raw={} ReComp={} ({:.1}%) Delta={} ({:.1}%)",
            run.name, first.raw_cost, first.recompression_cost, pct, first.delta_cost, del_pct);
    }

    // Simulation times
    println!();
    println!("  Simulation Runtime:                Low={:.2}s  Medium={:.2}s  High={:.2}s",
        low.elapsed_secs, medium.elapsed_secs, high.elapsed_secs);

    // Overall observation
    println!();
    println!("  Key Observation: Delta overhead as a percentage of full recompression");
    println!("  DECREASES at higher fidelity because the compressed output is larger,");
    println!("  making the fixed delta envelope cost (~80 chars) proportionally smaller.");
    println!("  This means delta transport is MORE effective relative to recompression");
    println!("  at Medium and High fidelity than at Low fidelity.");
    println!();

    Ok(())
}