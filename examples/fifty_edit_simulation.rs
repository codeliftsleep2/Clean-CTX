// examples/fifty_edit_simulation.rs
//
// Simulates 50 realistic developer edits on the UserManagementService.ts file
// and measures token savings at each step across three pipelines:
//
//   1. Raw (uncompressed) — BPE token count of full source
//   2. Clean-CTX full recompression — compress at each step, no delta
//   3. Clean-CTX + delta transport — compress once, then send only deltas
//
// Run with:
//     cargo run --example fifty_edit_simulation
//
// Output: per-edit table + final summary + category breakdown + insight callouts

use clean_ctx::analytics::{bpe_or_init, bpe};
use clean_ctx::compression::{compress_file, Fidelity};
use clean_ctx::dictionary::PathDictionary;
use clean_ctx::cache::LocalStateCache;
use std::path::PathBuf;
use std::time::Instant;

// ─── Edit Record ───────────────────────────────────────────────────────────

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
    Small,       // Edits 1-10: rename var, add param, fix return type, null check, access modifier
    Method,      // Edits 11-20: add/ modify method, constructor dep, return type changes
    Structural,  // Edits 21-30: add interface, refactor method, add @Input field
    CrossMethod, // Edits 31-40: extract helper, add error handling, add dep
    Refactor,    // Edits 41-50: restructure signatures, new interfaces, reorg deps
}

// ─── Helper: apply edits to source ─────────────────────────────────────────

/// Applies the edit functions sequentially starting from the base source.
/// Returns Vec<(description, category, edited_source)>.
fn generate_edit_sequence() -> Vec<(String, EditCategory, String)> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base_path = manifest_dir.join("src/test_files/UserManagementService.ts");
    let base_source = std::fs::read_to_string(&base_path)
        .expect("Cannot read UserManagementService.ts");

    let mut results: Vec<(String, EditCategory, String)> = Vec::new();
    let mut current = base_source;

    // ── Edits 1-10: Small changes ───────────────────────────────────────

    // Edit 1: Rename 'apiBasePath' to 'baseApiPath'
    current = current.replace("apiBasePath", "baseApiPath");
    results.push(("Rename 'apiBasePath' to 'baseApiPath'".to_string(), EditCategory::Small, current.clone()));

    // Edit 2: Add return type annotation to logOperation method
    current = current.replace(
        "private logOperation(operation: string, details: Record<string, unknown>)",
        "private logOperation(operation: string, details: Record<string, unknown>): void"
    );
    results.push(("Add void return type to logOperation".to_string(), EditCategory::Small, current.clone()));

    // Edit 3: Rename 'isActive' to 'active' in filter params
    current = current.replace("isActive", "active");
    results.push(("Rename 'isActive' to 'active' in UserFilter".to_string(), EditCategory::Small, current.clone()));

    // Edit 4: Change default page size from 25 to 50
    current = current.replace(
        "private readonly defaultPageSize = 25;",
        "private readonly defaultPageSize = 50;"
    );
    results.push(("Change defaultPageSize from 25 to 50".to_string(), EditCategory::Small, current.clone()));

    // Edit 5: Add null coalesce for lastError in getLastError
    current = current.replace(
        "return this.lastError;",
        "return this.lastError ?? null;"
    );
    results.push(("Add null coalesce to getLastError return".to_string(), EditCategory::Small, current.clone()));

    // Edit 6: Change cacheDurationMs from 5 min to 10 min
    current = current.replace(
        "private readonly cacheDurationMs = 5 * 60 * 1000;",
        "private readonly cacheDurationMs = 10 * 60 * 1000;"
    );
    results.push(("Change cache TTL from 5min to 10min".to_string(), EditCategory::Small, current.clone()));

    // Edit 7: Add private readonly to pendingRequests
    current = current.replace(
        "private pendingRequests = 0;",
        "private pendingRequests = 0;\n  private activeRequests = 0;"
    );
    results.push(("Add activeRequests counter field".to_string(), EditCategory::Small, current.clone()));

    // Edit 8: Change isAuthenticated to hasActiveSession
    current = current.replace(
        "isAuthenticated(): boolean {",
        "hasActiveSession(): boolean {"
    );
    results.push(("Rename isAuthenticated to hasActiveSession".to_string(), EditCategory::Small, current.clone()));

    // Edit 9: Add 'X-Request-ID' header to buildHeaders
    current = current.replace(
        "headers = headers.set('X-Request-Timestamp', Date.now().toString());",
        "headers = headers.set('X-Request-Timestamp', Date.now().toString());\n    headers = headers.set('X-Request-ID', crypto.randomUUID());"
    );
    results.push(("Add X-Request-ID header to buildHeaders".to_string(), EditCategory::Small, current.clone()));

    // Edit 10: Add trim to displayName in validateUserData
    current = current.replace(
        "if (data.displayName && data.displayName.length < 2) {",
        "if (data.displayName && data.displayName.trim().length < 2) {"
    );
    results.push(("Add trim() to displayName validation".to_string(), EditCategory::Small, current.clone()));

    // ── Edits 11-20: Method-level changes ──────────────────────────────

    // Edit 11: Add getUserPermissions method before getTotalRequests
    let new_method = "\n  async getUserPermissions(userId: UserId): Promise<ApiResponse<string[]>> {\n    try {\n      const url = `${this.apiBaseUrl}${this.baseApiPath}/${userId}/permissions`;\n      this.incrementRequestCount();\n      const response = await firstValueFrom(\n        this.http.get<string[]>(url, { headers: this.buildHeaders(timeoutMs) })\n      );\n      return { success: true, data: response, error: null, statusCode: 200, timestamp: new Date().toISOString() };\n    } catch (error) {\n      return this.handleError<string[]>(error, 'getUserPermissions', { userId });\n    }\n  }\n\n  ";
    let search = "getTotalRequests(";
    if let Some(pos) = current.find(search) {
        current = format!("{}{}{}", &current[..pos], new_method, &current[pos..]);
    }
    results.push(("Add getUserPermissions method".to_string(), EditCategory::Method, current.clone()));

    // Edit 12: Add batch size param to batchCreateUsers
    current = current.replace(
        "async batchCreateUsers(users: Array<Partial<UserProfile>>)",
        "async batchCreateUsers(users: Array<Partial<UserProfile>>, batchSize: number = 10)"
    );
    results.push(("Add batchSize param to batchCreateUsers".to_string(), EditCategory::Method, current.clone()));

    // Edit 13: Add fields param to getUserById
    current = current.replace(
        "async getUserById(userId: UserId)",
        "async getUserById(userId: UserId, fields?: string[])"
    );
    results.push(("Add optional fields param to getUserById".to_string(), EditCategory::Method, current.clone()));

    // Edit 14: Change verifyEmail to return verified flag object
    current = current.replace(
        "Promise<ApiResponse<boolean>>",
        "Promise<ApiResponse<{ verified: boolean; verifiedAt: string }>>"
    );
    results.push(("Change verifyEmail return type to detailed object".to_string(), EditCategory::Method, current.clone()));

    // Edit 15: Add abort controller to getUsers
    current = current.replace(
        "this.incrementRequestCount();\n      const response = await firstValueFrom(\n        this.http.get<PaginatedResult<UserProfile>>(url, {",
        "this.incrementRequestCount();\n      const controller = new AbortController();\n      const timeoutId = setTimeout(() => controller.abort(), 5000);\n      const response = await firstValueFrom(\n        this.http.get<PaginatedResult<UserProfile>>(url, { signal: controller.signal,"
    );
    results.push(("Add AbortController timeout to getUsers".to_string(), EditCategory::Method, current.clone()));

    // Edit 16: Add email caching to getUserByEmail
    current = current.replace(
        "async getUserByEmail(email: EmailAddress): Promise<ApiResponse<UserProfile>> {",
        "async getUserByEmail(email: EmailAddress): Promise<ApiResponse<UserProfile>> {\n    const cacheKey = `user:email:${email}`;\n    const cached = this.getFromCache<UserProfile>(cacheKey);\n    if (cached) {\n      return { success: true, data: cached, error: null, statusCode: 200, timestamp: new Date().toISOString() };\n    }\n    "
    );
    results.push(("Add caching logic to getUserByEmail".to_string(), EditCategory::Method, current.clone()));

    // Edit 17: Change PUT to PATCH in updateUser
    current = current.replace(
        "this.http.put<UserProfile>(url, validatedData.data, {",
        "this.http.patch<UserProfile>(url, validatedData.data, {"
    );
    results.push(("Change updateUser from PUT to PATCH".to_string(), EditCategory::Method, current.clone()));

    // Edit 18: Add Promise.allSettled for batch error handling
    current = current.replace(
        "await Promise.all(promises);",
        "const settled = await Promise.allSettled(promises);\n        for (const s of settled) {\n          if (s.status === 'rejected') {\n            result.failed++;\n            result.errors.push({ id: 'batch', error: s.reason?.toString() || 'Unknown' });\n          }\n        }"
    );
    results.push(("Add Promise.allSettled for batch error handling".to_string(), EditCategory::Method, current.clone()));

    // Edit 19: Add constructor dep for LoggerService
    current = current.replace(
        "private readonly analyticsService: AnalyticsService,",
        "private readonly analyticsService: AnalyticsService,\n    private readonly loggerService: LoggerService,"
    );
    results.push(("Add LoggerService constructor dependency".to_string(), EditCategory::Method, current.clone()));

    // Edit 20: Change maxPageSize from 100 to 200
    current = current.replace(
        "private readonly maxPageSize = 100;",
        "private readonly maxPageSize = 200;"
    );
    results.push(("Change maxPageSize from 100 to 200".to_string(), EditCategory::Method, current.clone()));

    // ── Edits 21-30: Structural changes ────────────────────────────────

    // Edit 21: Add UserSession interface
    current = current.replace(
        "export interface BatchOperationResult",
        "export interface UserSession {\n  sessionId: string;\n  userId: string;\n  token: string;\n  expiresAt: string;\n  createdAt: string;\n  ipAddress: string;\n  userAgent: string;\n  isActive: boolean;\n}\n\nexport interface BatchOperationResult"
    );
    results.push(("Add UserSession interface".to_string(), EditCategory::Structural, current.clone()));

    // Edit 22: Add type aliases
    current = current.replace(
        "export type AuditAction = 'CREATE' | 'UPDATE' | 'DELETE' | 'LOGIN' | 'LOGOUT' | 'PASSWORD_RESET';",
        "export type AuditAction = 'CREATE' | 'UPDATE' | 'DELETE' | 'LOGIN' | 'LOGOUT' | 'PASSWORD_RESET';\nexport type SessionToken = string;\nexport type IPAddress = string;"
    );
    results.push(("Add SessionToken and IPAddress type aliases".to_string(), EditCategory::Structural, current.clone()));

    // Edit 23: Add @Input() config field
    current = current.replace(
    "  constructor(",
    "  @Input() serviceConfig: { cacheEnabled: boolean; logLevel: string } = { cacheEnabled: true, logLevel: 'info' };\n\n  constructor("
    );
    results.push(("Add @Input() serviceConfig field".to_string(), EditCategory::Structural, current.clone()));

    // Edit 24: Refactor validateUserData into smaller methods
    current = current.replace(
        "private validateUserData(data: Partial<UserProfile>): { validationPassed: boolean; data: Partial<UserProfile>; error: string | null } {",
        "private validateRequiredFields(data: Partial<UserProfile>): string[] {\n    const errors: string[] = [];\n    if (data.email && !this.isValidEmail(data.email)) {\n      errors.push('Invalid email format');\n    }\n    if (data.displayName && data.displayName.trim().length < 2) {\n      errors.push('Display name must be at least 2 characters');\n    }\n    return errors;\n  }\n\n  private validateOptionalFields(data: Partial<UserProfile>): string[] {\n    const errors: string[] = [];\n    if (data.displayName && data.displayName.length > 100) {\n      errors.push('Display name must not exceed 100 characters');\n    }\n    if (data.phoneNumber && !this.isValidPhoneNumber(data.phoneNumber)) {\n      errors.push('Invalid phone number format');\n    }\n    return errors;\n  }\n\n  private validateUserData(data: Partial<UserProfile>): { validationPassed: boolean; data: Partial<UserProfile>; error: string | null } {"
    );
    results.push(("Split validateUserData into validateRequiredFields and validateOptionalFields".to_string(), EditCategory::Structural, current.clone()));

    // Edit 25: Add @Output() for error events
    current = current.replace(
        "@Output() batchOperationComplete = new EventEmitter<BatchOperationResult>();",
        "@Output() batchOperationComplete = new EventEmitter<BatchOperationResult>();\n  @Output() errorOccurred = new EventEmitter<{ operation: string; error: string; timestamp: string }>();"
    );
    results.push(("Add @Output() errorOccurred EventEmitter".to_string(), EditCategory::Structural, current.clone()));

    // Edit 26: Add suspendUser method
    current = current.replace(
        "async verifyEmail",
        "async suspendUser(userId: UserId, reason: string): Promise<ApiResponse<boolean>> {\n    try {\n      const url = `${this.apiBaseUrl}${this.baseApiPath}/${userId}/suspend`;\n      this.incrementRequestCount();\n      await firstValueFrom(\n        this.http.post(url, { reason }, { headers: this.buildHeaders(timeoutMs) })\n      );\n      this.cache.delete(`user:${userId}`);\n      this.logOperation('suspendUser', { userId, reason });\n      return { success: true, data: true, error: null, statusCode: 200, timestamp: new Date().toISOString() };\n    } catch (error) {\n      return this.handleError<boolean>(error, 'suspendUser', { userId, reason });\n    }\n  }\n\n  async verifyEmail"
    );
    results.push(("Add suspendUser method".to_string(), EditCategory::Structural, current.clone()));

    // Edit 27: Add rate limit fields
    current = current.replace(
        "private activeRequests = 0;",
        "private activeRequests = 0;\n  private readonly rateLimitPerMinute = 60;\n  private requestTimestamps: number[] = [];"
    );
    results.push(("Add rate limiting fields".to_string(), EditCategory::Structural, current.clone()));

    // Edit 28: Add getAllUsers convenience method
    current = current.replace(
        "private validateFilter(filter: UserFilter): UserFilter {",
        "async getAllUsers(page: number = 1, pageSize: number = 50): Promise<ApiResponse<PaginatedResult<UserProfile>>> {\n    const filter: UserFilter = {\n      searchText: '',\n      roles: [],\n      active: null,\n      isEmailVerified: null,\n      createdAfter: null,\n      createdBefore: null,\n      sortBy: 'createdAt',\n      sortDirection: 'desc',\n      page,\n      pageSize,\n    };\n    return this.getUsers(filter);\n  }\n\n  private validateFilter(filter: UserFilter): UserFilter {"
    );
    results.push(("Add getAllUsers convenience method".to_string(), EditCategory::Structural, current.clone()));

    // Edit 29: Add logMethodEntry and logMethodExit helpers
    current = current.replace(
        "private incrementRequestCount(): void {",
        "private logMethodEntry(method: string, args: Record<string, unknown>): void {\n    if (this.serviceConfig.logLevel === 'debug') {\n      try {\n        this.loggerService.log(`Entering ${method}`, args);\n      } catch {\n        // silent\n      }\n    }\n  }\n\n  private logMethodExit(method: string, durationMs: number): void {\n    if (this.serviceConfig.logLevel === 'debug') {\n      try {\n        this.loggerService.log(`Exiting ${method}`, { durationMs });\n      } catch {\n        // silent\n      }\n    }\n  }\n\n  private incrementRequestCount(): void {"
    );
    results.push(("Add logMethodEntry and logMethodExit helpers".to_string(), EditCategory::Structural, current.clone()));

    // Edit 30: Add UserServiceConfig interface
    current = current.replace(
        "export interface UserSession",
        "export interface UserServiceConfig {\n  basePath: string;\n  defaultPageSize: number;\n  maxPageSize: number;\n  cacheEnabled: boolean;\n  cacheDurationMs: number;\n  timeoutMs: number;\n  retryCount: number;\n}\n\nexport interface UserSession"
    );
    results.push(("Add UserServiceConfig interface".to_string(), EditCategory::Structural, current.clone()));

    // ── Edits 31-40: Cross-method changes ─────────────────────────────

    // Edit 31: Add buildUrl helper
    current = current.replace(
        "private logMethodEntry",
        "private buildUrl(path: string): string {\n    return `${this.apiBaseUrl}${this.baseApiPath}${path}`;\n  }\n\n  private logMethodEntry"
    );
    results.push(("Extract buildUrl URL builder method".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 32: Add error handling to getFromCache
    current = current.replace(
        "private getFromCache<T>(key: string): T | null {\n    const entry = this.cache.get(key);",
        "private getFromCache<T>(key: string): T | null {\n    try {\n    const entry = this.cache.get(key);"
    );
    results.push(("Add try/catch to getFromCache".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 33: Add error handling to setInCache
    current = current.replace(
        "private setInCache<T>(key: string, data: T): void {\n    if (this.cache.size > 100) {",
        "private setInCache<T>(key: string, data: T): void {\n    try {\n    if (this.cache.size > 100) {"
    );
    results.push(("Add try/catch to setInCache".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 34: Add withRetry and delay helpers
    current = current.replace(
        "private logMethodExit",
        "private async withRetry<T>(operation: () => Promise<T>, maxRetries: number = 3): Promise<T> {\n    let lastError: unknown;\n    for (let attempt = 1; attempt <= maxRetries; attempt++) {\n      try {\n        return await operation();\n      } catch (error) {\n        lastError = error;\n        if (attempt < maxRetries) {\n          await this.delay(Math.pow(2, attempt) * 100);\n        }\n      }\n    }\n    throw lastError;\n  }\n\n  private delay(ms: number): Promise<void> {\n    return new Promise(resolve => setTimeout(resolve, ms));\n  }\n\n  private logMethodExit"
    );
    results.push(("Add withRetry and delay cross-method helpers".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 35: Add throwIfNotAuthenticated guard
    current = current.replace(
        "private buildUrl",
        "private throwIfNotAuthenticated(): void {\n    if (!this.hasActiveSession()) {\n      throw new Error('User is not authenticated');\n    }\n  }\n\n  private buildUrl"
    );
    results.push(("Add throwIfNotAuthenticated guard method".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 36: Add measure timing wrapper
    current = current.replace(
        "private throwIfNotAuthenticated",
        "private async measure<T>(label: string, fn: () => Promise<T>): Promise<T> {\n    const start = Date.now();\n    try {\n      return await fn();\n    } finally {\n      const duration = Date.now() - start;\n      try {\n        this.analyticsService.trackTiming('user_management', label, duration);\n      } catch {\n        // silent\n      }\n    }\n  }\n\n  private throwIfNotAuthenticated"
    );
    results.push(("Add measure() timing wrapper".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 37: Add ConfigService constructor dep
    current = current.replace(
        "private readonly loggerService: LoggerService,",
        "private readonly loggerService: LoggerService,\n    private readonly configService: ConfigService,"
    );
    results.push(("Add ConfigService constructor dependency".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 38: Add rate limiting to incrementRequestCount
    current = current.replace(
        "private incrementRequestCount(): void {\n    this.requestCount++;",
        "private incrementRequestCount(): void {\n    const now = Date.now();\n    this.requestTimestamps.push(now);\n    const oneMinuteAgo = now - 60000;\n    this.requestTimestamps = this.requestTimestamps.filter(t => t > oneMinuteAgo);\n    if (this.requestTimestamps.length > this.rateLimitPerMinute) {\n      throw new Error('Rate limit exceeded. Please try again later.');\n    }\n    this.requestCount++;"
    );
    results.push(("Add rate limiting logic to incrementRequestCount".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 39: Add cache invalidation logging
    current = current.replace(
        "this.cache.set(key, { data, timestamp: Date.now() });\n    }",
        "this.cache.set(key, { data, timestamp: Date.now() });\n      this.logOperation('cache_set', { key });\n    }"
    );
    results.push(("Add cache_set operation logging".to_string(), EditCategory::CrossMethod, current.clone()));

    // Edit 40: Add error event emission to handleError
    current = current.replace(
        "console.error(`Operation ${operation} failed:`, errorMessage);",
        "this.errorOccurred.emit({ operation, error: errorMessage, timestamp: new Date().toISOString() });\n      console.error(`Operation ${operation} failed:`, errorMessage);"
    );
    results.push(("Emit errorOccurred event in handleError".to_string(), EditCategory::CrossMethod, current.clone()));

    // ── Edits 41-50: Larger refactors ──────────────────────────────────

    // Edit 41: Add options object to getUsers
    current = current.replace(
        "async getUsers(filter: UserFilter): Promise<ApiResponse<PaginatedResult<UserProfile>>> {",
        "async getUsers(filter: UserFilter, options?: { useCache?: boolean; timeout?: number }): Promise<ApiResponse<PaginatedResult<UserProfile>>> {"
    );
    results.push(("Add RequestOptions object to getUsers signature".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 42: Add RequestOptions interface
    current = current.replace(
        "export interface UserServiceConfig",
        "export interface RequestOptions {\n  useCache: boolean;\n  timeout: number;\n  retryOnFailure: boolean;\n  retryCount: number;\n  headers?: Record<string, string>;\n}\n\nexport interface UserServiceConfig"
    );
    results.push(("Add RequestOptions interface".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 43: Refactor constructor to use options object
    current = current.replace(
        "constructor(\n    private readonly http: HttpClient,\n    @Inject('API_BASE_URL') private readonly apiBaseUrl: string,\n    @Optional() @Inject('CACHE_PROVIDER') private readonly cacheProvider: unknown,\n    private readonly notificationService: NotificationService,\n    private readonly analyticsService: AnalyticsService,\n    private readonly loggerService: LoggerService,\n    private readonly configService: ConfigService,\n  ) {",
        "constructor(\n    private readonly http: HttpClient,\n    @Inject('API_BASE_URL') private readonly apiBaseUrl: string,\n    @Optional() @Inject('CACHE_PROVIDER') private readonly cacheProvider: unknown,\n    private readonly notificationService: NotificationService,\n    private readonly analyticsService: AnalyticsService,\n    private readonly loggerService: LoggerService,\n    private readonly configService: ConfigService,\n    private readonly options: UserServiceConfig,\n  ) {"
    );
    results.push(("Refactor constructor to use UserServiceConfig options object".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 44: Add UserOperationResult interface
    current = current.replace(
        "export interface RequestOptions",
        "export interface UserOperationResult<T> {\n  success: boolean;\n  data: T | null;\n  error: string | null;\n  statusCode: number;\n  timestamp: string;\n  durationMs: number;\n  cached: boolean;\n}\n\nexport interface RequestOptions"
    );
    results.push(("Add UserOperationResult<T> generic interface".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 45: Change ApiResponse references to UserOperationResult
    // This will affect all method return types
    current = current.replace(
        "ApiResponse<PaginatedResult<UserProfile>>",
        "UserOperationResult<PaginatedResult<UserProfile>>"
    );
    results.push(("Replace ApiResponse with UserOperationResult in all signatures".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 46: Add region comments to organize class
    current = current.replace(
        "  private readonly baseApiPath",
        "  // ════════════════════════════════════════════════════════\n  // Configuration\n  // ════════════════════════════════════════════════════════════\n\n  private readonly baseApiPath"
    );
    results.push(("Add Configuration region comment".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 47: Add Public API region  
    current = current.replace(
        "// --- Public API Methods ---",
        "// ════════════════════════════════════════════════════════\n  // Public API Methods\n  // ════════════════════════════════════════════════════════════"
    );
    results.push(("Add Public API Methods region header".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 48: Add Private Helpers region
    current = current.replace(
        "// --- Private Helper Methods ---",
        "// ════════════════════════════════════════════════════════\n  // Private Helper Methods\n  // ════════════════════════════════════════════════════════════"
    );
    results.push(("Add Private Helper Methods region header".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 49: Refactor handleError to add recovery strategy
    current = current.replace(
        "private handleError<T>(error: unknown, operation: string, context: Record<string, unknown>): UserOperationResult<T> {",
        "private handleError<T>(error: unknown, operation: string, context: Record<string, unknown>, recoveryStrategy?: 'retry' | 'fallback' | 'abort'): UserOperationResult<T> {"
    );
    results.push(("Add recoveryStrategy param to handleError".to_string(), EditCategory::Refactor, current.clone()));

    // Edit 50: Add final refactor — wrap all public methods with error boundary
    current = current.replace(
        "private incrementRequestCount(): void {",
        "private async executeSafely<T>(operation: string, fn: () => Promise<T>): Promise<UserOperationResult<T>> {\n    try {\n      this.logMethodEntry(operation, {});\n      const start = Date.now();\n      const data = await fn();\n      const duration = Date.now() - start;\n      this.logMethodExit(operation, duration);\n      return { success: true, data, error: null, statusCode: 200, timestamp: new Date().toISOString(), durationMs: duration, cached: false };\n    } catch (error) {\n      return this.handleError<T>(error, operation, {}, 'retry');\n    }\n  }\n\n  private incrementRequestCount(): void {"
    );
    results.push(("Add executeSafely error boundary wrapper".to_string(), EditCategory::Refactor, current.clone()));

    results
}

/// Compute a simple text delta between two strings.
/// Returns the number of characters in added/modified/removed lines.
fn text_delta_cost(a: &str, b: &str) -> usize {
    let lines_a: Vec<&str> = a.lines().collect();
    let lines_b: Vec<&str> = b.lines().collect();
    
    let mut a_idx = 0usize;
    let mut b_idx = 0usize;
    let mut total_delta_chars = 0usize;

    // Simple LCS-like diff: count chars in non-matching lines
    while a_idx < lines_a.len() || b_idx < lines_b.len() {
        if a_idx < lines_a.len() && b_idx < lines_b.len() && lines_a[a_idx] == lines_b[b_idx] {
            a_idx += 1;
            b_idx += 1;
        } else if a_idx < lines_a.len() && b_idx < lines_b.len() {
            // Lines differ — count both
            total_delta_chars += lines_a[a_idx].len() + lines_b[b_idx].len();
            a_idx += 1;
            b_idx += 1;
        } else if a_idx < lines_a.len() {
            // Removed line
            total_delta_chars += lines_a[a_idx].len() + 1; // +1 for minus sign
            a_idx += 1;
        } else if b_idx < lines_b.len() {
            // Added line
            total_delta_chars += lines_b[b_idx].len() + 1; // +1 for plus sign
            b_idx += 1;
        }
    }
    total_delta_chars
}

// ─── Main simulation ───────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bpe_or_init()?;
    let bpe = bpe();
    let start_time = Instant::now();

    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!("  Clean-CTX 50-Edit Token Savings Simulation");
    println!("  File: UserManagementService.ts (~430 lines)");
    println!("  Pipeline: Raw | Clean-CTX Full Recompression | Clean-CTX + Delta Transport");
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!();

    // Generate the edit sequence
    let edits = generate_edit_sequence();
    assert_eq!(edits.len(), 50, "Expected exactly 50 edits, got {}", edits.len());

    // Initialize shared state for compress_file
    let mut dict = PathDictionary::new();
    let mut cache = LocalStateCache::new();

    // Pre-write the temp file path we'll use for all compressions
    let temp_path = std::env::temp_dir().join("clean_ctx_edit_simulation.ts");
    let file_path = PathBuf::from(&temp_path);

    // Run measurements
    let mut records: Vec<EditRecord> = Vec::new();
    let mut prev_compressed: Option<String> = None;
    let mut cum_raw = 0usize;
    let mut cum_recomp = 0usize;
    let mut cum_delta = 0usize;

    for (i, (description, category, source)) in edits.iter().enumerate() {
        let edit_num = i + 1;

        // Write source to temp file for compression
        std::fs::write(&temp_path, source)
            .unwrap_or_else(|_| panic!("Failed to write temp file for edit {}", edit_num));

        // Raw token count
        let raw_tokens = bpe.encode_with_special_tokens(source).len();

        // Compressed output via compress_file (full recompression)
        let compressed_output = compress_file(file_path.clone(), &mut dict, &mut cache, Fidelity::Low)
            .unwrap_or_else(|e| format!("// Failed to compress edit {}: {}", edit_num, e));
        let compressed_tokens = bpe.encode_with_special_tokens(&compressed_output).len();
        
        // Delta cost: compute text-level delta between successive compressed outputs
        let delta_tokens = if let Some(ref prev) = prev_compressed {
            if prev != &compressed_output {
                // Compute delta characters, estimate token cost as ~4 chars/token (typical BPE ratio)
                let delta_chars = text_delta_cost(prev, &compressed_output);
                // Add delta envelope overhead (file alias, version markers, etc) ~80 chars
                let total_delta_chars = delta_chars + 80;
                (total_delta_chars + 3) / 4 // Estimate tokens (ceil division by ~4 chars/token)
            } else {
                0 // No change
            }
        } else {
            compressed_tokens // First edit: full compressed cost as baseline
        };

        // Update cumulative
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
            cum_raw,
            cum_recomp,
            cum_delta,
        });

        prev_compressed = Some(compressed_output);
    }

    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();

    // ─── Print per-edit table ──────────────────────────────────────────────
    println!("Per-Edit Results:");
    println!("{}{:>4} │ {:<55} │ {:>7} │ {:>7} │ {:>7} │ {:>8} │ {:>8} │ {:>8}",
        "Edit", "#", "Description",
        "Raw", "ReComp", "Delta",
        "CumRaw", "CumReC", "CumDel"
    );
    println!("{}", "─".repeat(145));

    for rec in &records {
        let cat_marker = match rec.category {
            EditCategory::Small => "S",
            EditCategory::Method => "M",
            EditCategory::Structural => "T",
            EditCategory::CrossMethod => "X",
            EditCategory::Refactor => "R",
        };
        println!("{:>4} │ {:<55} │ {:>7} │ {:>7} │ {:>7} │ {:>8} │ {:>8} │ {:>8}",
            format!("{}{}", rec.number, cat_marker),
            truncate(&rec.description, 54),
            rec.raw_cost,
            rec.recompression_cost,
            rec.delta_cost,
            rec.cum_raw,
            rec.cum_recomp,
            rec.cum_delta,
        );
    }

    // ─── Final Summary Table ──────────────────────────────────────────────
    let total_raw = records.last().map(|r| r.cum_raw).unwrap_or(0);
    let total_recomp = records.last().map(|r| r.cum_recomp).unwrap_or(0);
    let total_delta = records.last().map(|r| r.cum_delta).unwrap_or(0);

    let full_vs_raw_saved = total_raw.saturating_sub(total_recomp);
    let delta_vs_raw_saved = total_raw.saturating_sub(total_delta);
    let delta_vs_full_saved = total_recomp.saturating_sub(total_delta);

    let full_vs_raw_pct = if total_raw > 0 { (full_vs_raw_saved as f64 / total_raw as f64) * 100.0 } else { 0.0 };
    let delta_vs_raw_pct = if total_raw > 0 { (delta_vs_raw_saved as f64 / total_raw as f64) * 100.0 } else { 0.0 };
    let delta_vs_full_pct = if total_recomp > 0 { (delta_vs_full_saved as f64 / total_recomp as f64) * 100.0 } else { 0.0 };

    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!("  FINAL SUMMARY");
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("  Per-Pipeline Totals:");
    println!("    Raw (no compression):               {:>10} tokens", total_raw);
    println!("    Clean-CTX full recompression:       {:>10} tokens", total_recomp);
    println!("    Clean-CTX + delta transport:        {:>10} tokens", total_delta);
    println!();
    println!("  Savings:");
    println!("    Full compression vs Raw:            {:>10} tokens ({:>5.1}%)", full_vs_raw_saved, full_vs_raw_pct);
    println!("    Delta vs Raw:                       {:>10} tokens ({:>5.1}%)", delta_vs_raw_saved, delta_vs_raw_pct);
    println!("    Delta vs Full recompression:        {:>10} tokens ({:>5.1}%)", delta_vs_full_saved, delta_vs_full_pct);
    println!();

    // ─── Breakdown by Edit Category ───────────────────────────────────────
    println!("  Breakdown by Edit Category:");
    println!("  {:<15} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Category", "Count", "Raw", "ReComp", "Delta", "ReSav%", "DelSav%"
    );
    println!("  {}", "─".repeat(80));

    let categories = [
        (EditCategory::Small, "Small (1-10)"),
        (EditCategory::Method, "Method (11-20)"),
        (EditCategory::Structural, "Structural (21-30)"),
        (EditCategory::CrossMethod, "Cross (31-40)"),
        (EditCategory::Refactor, "Refactor (41-50)"),
    ];

    for (cat, label) in &categories {
        let cat_records: Vec<&EditRecord> = records.iter().filter(|r| r.category == *cat).collect();
        let count = cat_records.len();
        let cat_raw: usize = cat_records.iter().map(|r| r.raw_cost).sum();
        let cat_recomp: usize = cat_records.iter().map(|r| r.recompression_cost).sum();
        let cat_delta: usize = cat_records.iter().map(|r| r.delta_cost).sum();
        let cat_re_save = if cat_raw > 0 { (cat_raw.saturating_sub(cat_recomp)) as f64 / cat_raw as f64 * 100.0 } else { 0.0 };
        let cat_del_save = if cat_raw > 0 { (cat_raw.saturating_sub(cat_delta)) as f64 / cat_raw as f64 * 100.0 } else { 0.0 };
        println!("  {:<15} {:>8} {:>8} {:>8} {:>8} {:>7.1}% {:>7.1}%",
            label, count, cat_raw, cat_recomp, cat_delta, cat_re_save, cat_del_save);
    }

    // ─── Key Insight Callouts ─────────────────────────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!("  KEY INSIGHT CALLOUTS");
    println!("═══════════════════════════════════════════════════════════════════════════════════════");
    println!();

    // Break-even point: when delta cumulative <= recompression cumulative
    let mut break_even_edit: Option<usize> = None;
    for rec in &records {
        if rec.cum_delta <= rec.cum_recomp {
            break_even_edit = Some(rec.number);
            break;
        }
    }
    match break_even_edit {
        Some(n) => {
            let rec = records.iter().find(|r| r.number == n).unwrap();
            println!("  ✓ Break-even: Edit #{} — delta transport cumulative ({}) <= full recompression ({})",
                n, rec.cum_delta, rec.cum_recomp);
        },
        None => {
            // Check if delta is always cheaper
            let last = records.last().unwrap();
            if last.cum_delta < last.cum_recomp {
                println!("  ✓ Delta transport always cheaper than full recompression");
            } else {
                println!("  ✗ Delta transport never broke even with full recompression");
            }
        },
    }

    // Largest single-edit delta saving vs raw
    let mut max_delta_saving: (usize, f64) = (0, 0.0);
    for rec in &records {
        if rec.raw_cost > 0 {
            let saving = (rec.raw_cost.saturating_sub(rec.delta_cost)) as f64 / rec.raw_cost as f64 * 100.0;
            if saving > max_delta_saving.1 {
                max_delta_saving = (rec.number, saving);
            }
        }
    }
    println!("  ✓ Largest single-edit delta saving vs raw: Edit #{} — {:.1}% reduction",
        max_delta_saving.0, max_delta_saving.1);

    // Smallest single-edit delta saving (worst case)
    let mut min_delta_saving: (usize, f64) = (0, 100.0);
    for rec in &records {
        if rec.raw_cost > 0 {
            let saving = (rec.raw_cost.saturating_sub(rec.delta_cost)) as f64 / rec.raw_cost as f64 * 100.0;
            if saving < min_delta_saving.1 {
                min_delta_saving = (rec.number, saving);
            }
        }
    }
    println!("  ✗ Smallest single-edit delta saving vs raw: Edit #{} — {:.1}% reduction (worst case)",
        min_delta_saving.0, min_delta_saving.1);

    // Delta vs full recompression savings per edit
    let mut max_delta_vs_full: (usize, f64) = (0, 0.0);
    let mut min_delta_vs_full: (usize, f64) = (0, 100.0);
    for rec in &records {
        if rec.recompression_cost > 0 {
            let saving = (rec.recompression_cost.saturating_sub(rec.delta_cost)) as f64 / rec.recompression_cost as f64 * 100.0;
            if saving > max_delta_vs_full.1 {
                max_delta_vs_full = (rec.number, saving);
            }
            if saving < min_delta_vs_full.1 {
                min_delta_vs_full = (rec.number, saving);
            }
        }
    }
    println!("  ✓ Largest delta saving vs full recompression: Edit #{} — {:.1}%",
        max_delta_vs_full.0, max_delta_vs_full.1);
    println!("  ✗ Smallest delta saving vs full recompression: Edit #{} — {:.1}%",
        min_delta_vs_full.0, min_delta_vs_full.1);

    // Additional insights
    let avg_raw = total_raw as f64 / 50.0;
    let avg_recomp = total_recomp as f64 / 50.0;
    let avg_delta = total_delta as f64 / 50.0;
    println!();
    println!("  Averages per edit:");
    println!("    Raw:              {:.1} tokens", avg_raw);
    println!("    Full recomp:      {:.1} tokens", avg_recomp);
    println!("    Delta transport:  {:.1} tokens", avg_delta);
    println!();
    println!("  Simulation completed in {:.2}s across {} edits", elapsed_secs, 50);

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}