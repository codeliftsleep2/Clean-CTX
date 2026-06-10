// src/test_files/UserManagementService.ts
//
// Test fixture: Angular @Injectable service for token savings simulation.
// ~430 lines of realistic Angular service code with business logic.

import { Injectable, Inject, Optional, Output, EventEmitter } from '@angular/core';
import { HttpClient, HttpParams, HttpHeaders } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

// --- Interface Definitions ---

export interface UserProfile {
  id: string;
  email: string;
  displayName: string;
  avatarUrl: string;
  phoneNumber: string;
  dateOfBirth: string;
  isActive: boolean;
  isEmailVerified: boolean;
  roles: string[];
  permissions: string[];
  createdAt: string;
  updatedAt: string;
  lastLoginAt: string | null;
  loginAttempts: number;
  preferences: UserPreferences;
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'system';
  language: string;
  timezone: string;
  notificationsEnabled: boolean;
  emailNotifications: boolean;
  smsNotifications: boolean;
  twoFactorEnabled: boolean;
  sessionTimeoutMinutes: number;
}

export interface UserFilter {
  searchText: string;
  roles: string[];
  isActive: boolean | null;
  isEmailVerified: boolean | null;
  createdAfter: string | null;
  createdBefore: string | null;
  sortBy: string;
  sortDirection: SortDirection;
  page: number;
  pageSize: number;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
  statusCode: number;
  timestamp: string;
}

export interface BatchOperationResult {
  succeeded: number;
  failed: number;
  errors: Array<{ id: string; error: string }>;
  totalProcessed: number;
  durationMs: number;
}

// --- Type Aliases ---

export type UserId = string;
export type EmailAddress = string;
export type SortDirection = 'asc' | 'desc';
export type UserStatus = 'active' | 'inactive' | 'suspended' | 'pending';
export type AuditAction = 'CREATE' | 'UPDATE' | 'DELETE' | 'LOGIN' | 'LOGOUT' | 'PASSWORD_RESET';

@Injectable({
  providedIn: 'root',
})
export class UserManagementService {
  @Output() userCreated = new EventEmitter<UserProfile>();
  @Output() userUpdated = new EventEmitter<UserProfile>();
  @Output() userDeleted = new EventEmitter<string>();
  @Output() batchOperationComplete = new EventEmitter<BatchOperationResult>();

  private readonly apiBasePath = '/api/v1/users';
  private readonly defaultPageSize = 25;
  private readonly maxPageSize = 100;
  private readonly cacheDurationMs = 5 * 60 * 1000;
  private readonly cache = new Map<string, { data: unknown; timestamp: number }>();

  private pendingRequests = 0;
  private lastError: string | null = null;
  private requestCount = 0;

  constructor(
    private readonly http: HttpClient,
    @Inject('API_BASE_URL') private readonly apiBaseUrl: string,
    @Optional() @Inject('CACHE_PROVIDER') private readonly cacheProvider: unknown,
    private readonly notificationService: NotificationService,
    private readonly analyticsService: AnalyticsService,
  ) {
    this.apiBaseUrl = apiBaseUrl.replace(/\/+$/, '');
  }

  // --- Public API Methods ---

  async getUserById(userId: UserId): Promise<ApiResponse<UserProfile>> {
    const cacheKey = `user:${userId}`;
    const cached = this.getFromCache<UserProfile>(cacheKey);
    if (cached) {
      return {
        success: true,
        data: cached,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    }

    try {
      this.incrementRequestCount();
      const url = `${this.apiBaseUrl}${this.apiBasePath}/${userId}`;
      const response = await firstValueFrom(
        this.http.get<UserProfile>(url, {
          headers: this.buildHeaders(),
        })
      );
      this.setInCache(cacheKey, response);
      return {
        success: true,
        data: response,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<UserProfile>(error, 'getUserById', { userId });
    }
  }

  async getUsers(filter: UserFilter): Promise<ApiResponse<PaginatedResult<UserProfile>>> {
    try {
      const validatedFilter = this.validateFilter(filter);
      const params = this.buildFilterParams(validatedFilter);
      const url = `${this.apiBaseUrl}${this.apiBasePath}`;

      this.incrementRequestCount();
      const response = await firstValueFrom(
        this.http.get<PaginatedResult<UserProfile>>(url, {
          params,
          headers: this.buildHeaders(),
        })
      );

      this.logOperation('getUsers', { filter: validatedFilter, resultCount: response.items.length });
      return {
        success: true,
        data: response,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<PaginatedResult<UserProfile>>(error, 'getUsers', { filter });
    }
  }

  async createUser(userData: Partial<UserProfile>): Promise<ApiResponse<UserProfile>> {
    try {
      const validatedData = this.validateUserData(userData);
      if (!validatedData.isValid) {
        return {
          success: false,
          data: null,
          error: validatedData.error,
          statusCode: 400,
          timestamp: new Date().toISOString(),
        };
      }

      const url = `${this.apiBaseUrl}${this.apiBasePath}`;
      this.incrementRequestCount();
      const response = await firstValueFrom(
        this.http.post<UserProfile>(url, validatedData.data, {
          headers: this.buildHeaders(),
        })
      );

      this.invalidateUserCache();
      this.userCreated.emit(response);
      this.logOperation('createUser', { userId: response.id });
      return {
        success: true,
        data: response,
        error: null,
        statusCode: 201,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<UserProfile>(error, 'createUser', { userData });
    }
  }

  async updateUser(userId: UserId, updates: Partial<UserProfile>): Promise<ApiResponse<UserProfile>> {
    try {
      const validatedData = this.validateUserData(updates);
      if (!validatedData.isValid) {
        return {
          success: false,
          data: null,
          error: validatedData.error,
          statusCode: 400,
          timestamp: new Date().toISOString(),
        };
      }

      const url = `${this.apiBaseUrl}${this.apiBasePath}/${userId}`;
      this.incrementRequestCount();
      const response = await firstValueFrom(
        this.http.put<UserProfile>(url, validatedData.data, {
          headers: this.buildHeaders(),
        })
      );

      this.cache.delete(`user:${userId}`);
      this.userUpdated.emit(response);
      this.logOperation('updateUser', { userId });
      return {
        success: true,
        data: response,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<UserProfile>(error, 'updateUser', { userId, updates });
    }
  }

  async deleteUser(userId: UserId): Promise<ApiResponse<boolean>> {
    try {
      const url = `${this.apiBaseUrl}${this.apiBasePath}/${userId}`;
      this.incrementRequestCount();
      await firstValueFrom(
        this.http.delete(url, {
          headers: this.buildHeaders(),
        })
      );

      this.cache.delete(`user:${userId}`);
      this.userDeleted.emit(userId);
      this.logOperation('deleteUser', { userId });
      return {
        success: true,
        data: true,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<boolean>(error, 'deleteUser', { userId });
    }
  }

  async batchCreateUsers(users: Array<Partial<UserProfile>>): Promise<BatchOperationResult> {
    const result: BatchOperationResult = {
      succeeded: 0,
      failed: 0,
      errors: [],
      totalProcessed: 0,
      durationMs: 0,
    };

    const startTime = Date.now();
    const batches = this.chunkArray(users, 10);

    for (const batch of batches) {
      const promises = batch.map(async (userData) => {
        try {
          const response = await this.createUser(userData);
          if (response.success) {
            result.succeeded++;
          } else {
            result.failed++;
            result.errors.push({ id: 'unknown', error: response.error || 'Unknown error' });
          }
        } catch (error) {
          result.failed++;
          result.errors.push({ id: 'unknown', error: String(error) });
        }
      });
      await Promise.all(promises);
    }

    result.totalProcessed = result.succeeded + result.failed;
    result.durationMs = Date.now() - startTime;
    this.batchOperationComplete.emit(result);
    return result;
  }

  async getUserByEmail(email: EmailAddress): Promise<ApiResponse<UserProfile>> {
    if (!this.isValidEmail(email)) {
      return {
        success: false,
        data: null,
        error: 'Invalid email format',
        statusCode: 400,
        timestamp: new Date().toISOString(),
      };
    }

    try {
      const params = new HttpParams().set('email', email);
      const url = `${this.apiBaseUrl}${this.apiBasePath}/by-email`;
      this.incrementRequestCount();
      const response = await firstValueFrom(
        this.http.get<UserProfile>(url, {
          params,
          headers: this.buildHeaders(),
        })
      );
      return {
        success: true,
        data: response,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<UserProfile>(error, 'getUserByEmail', { email });
    }
  }

  async verifyEmail(userId: UserId): Promise<ApiResponse<boolean>> {
    try {
      const url = `${this.apiBaseUrl}${this.apiBasePath}/${userId}/verify-email`;
      this.incrementRequestCount();
      await firstValueFrom(
        this.http.post(url, null, { headers: this.buildHeaders() })
      );
      this.cache.delete(`user:${userId}`);
      return {
        success: true,
        data: true,
        error: null,
        statusCode: 200,
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return this.handleError<boolean>(error, 'verifyEmail', { userId });
    }
  }

  async getUserCount(isActive: boolean | null = null): Promise<number> {
    try {
      let params = new HttpParams();
      if (isActive !== null) {
        params = params.set('isActive', String(isActive));
      }
      params = params.set('countOnly', 'true');
      const url = `${this.apiBaseUrl}${this.apiBasePath}/count`;
      this.incrementRequestCount();
      const response = await firstValueFrom(
        this.http.get<{ count: number }>(url, { params, headers: this.buildHeaders() })
      );
      return response.count;
    } catch (error) {
      this.lastError = `getUserCount failed: ${error}`;
      return 0;
    }
  }

  isAuthenticated(): boolean {
    return this.pendingRequests >= 0;
  }

  getLastError(): string | null {
    return this.lastError;
  }

  getTotalRequests(): number {
    return this.requestCount;
  }

  // --- Private Helper Methods ---

  private buildHeaders(): HttpHeaders {
    let headers = new HttpHeaders();
    headers = headers.set('Content-Type', 'application/json');
    headers = headers.set('Accept', 'application/json');
    headers = headers.set('X-Request-Timestamp', Date.now().toString());
    return headers;
  }

  private buildFilterParams(filter: UserFilter): HttpParams {
    let params = new HttpParams();
    if (filter.searchText) {
      params = params.set('search', filter.searchText.trim());
    }
    if (filter.roles && filter.roles.length > 0) {
      params = params.set('roles', filter.roles.join(','));
    }
    if (filter.isActive !== null) {
      params = params.set('isActive', String(filter.isActive));
    }
    if (filter.isEmailVerified !== null) {
      params = params.set('isEmailVerified', String(filter.isEmailVerified));
    }
    if (filter.createdAfter) {
      params = params.set('createdAfter', filter.createdAfter);
    }
    if (filter.createdBefore) {
      params = params.set('createdBefore', filter.createdBefore);
    }
    params = params.set('sortBy', filter.sortBy || 'createdAt');
    params = params.set('sortDirection', filter.sortDirection || 'desc');
    params = params.set('page', String(Math.max(1, filter.page)));
    params = params.set('pageSize', String(Math.min(this.maxPageSize, Math.max(1, filter.pageSize))));
    return params;
  }

  private validateUserData(data: Partial<UserProfile>): { isValid: boolean; data: Partial<UserProfile>; error: string | null } {
    const errors: string[] = [];
    if (data.email && !this.isValidEmail(data.email)) {
      errors.push('Invalid email format');
    }
    if (data.displayName && data.displayName.length < 2) {
      errors.push('Display name must be at least 2 characters');
    }
    if (data.displayName && data.displayName.length > 100) {
      errors.push('Display name must not exceed 100 characters');
    }
    if (data.phoneNumber && !this.isValidPhoneNumber(data.phoneNumber)) {
      errors.push('Invalid phone number format');
    }
    if (errors.length > 0) {
      return { isValid: false, data, error: errors.join('; ') };
    }
    return { isValid: true, data, error: null };
  }

  private validateFilter(filter: UserFilter): UserFilter {
    return {
      ...filter,
      page: Math.max(1, filter.page || 1),
      pageSize: Math.min(this.maxPageSize, Math.max(1, filter.pageSize || this.defaultPageSize)),
      sortBy: filter.sortBy || 'createdAt',
      sortDirection: filter.sortDirection || 'desc',
    };
  }

  private isValidEmail(email: string): boolean {
    if (!email || email.length > 254) {
      return false;
    }
    const atIndex = email.indexOf('@');
    if (atIndex < 1 || atIndex !== email.lastIndexOf('@')) {
      return false;
    }
    const localPart = email.substring(0, atIndex);
    const domainPart = email.substring(atIndex + 1);
    if (localPart.length > 64 || domainPart.length < 4) {
      return false;
    }
    if (!domainPart.includes('.')) {
      return false;
    }
    return true;
  }

  private isValidPhoneNumber(phone: string): boolean {
    const cleaned = phone.replace(/[\s\-\(\)\+]/g, '');
    if (cleaned.length < 10 || cleaned.length > 15) {
      return false;
    }
    return /^\d+$/.test(cleaned);
  }

  private chunkArray<T>(array: T[], size: number): T[][] {
    const chunks: T[][] = [];
    for (let i = 0; i < array.length; i += size) {
      chunks.push(array.slice(i, i + size));
    }
    return chunks;
  }

  private getFromCache<T>(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) {
      return null;
    }
    if (Date.now() - entry.timestamp > this.cacheDurationMs) {
      this.cache.delete(key);
      return null;
    }
    return entry.data as T;
  }

  private setInCache<T>(key: string, data: T): void {
    if (this.cache.size > 100) {
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }
    this.cache.set(key, { data, timestamp: Date.now() });
  }

  private invalidateUserCache(): void {
    for (const key of this.cache.keys()) {
      if (key.startsWith('user:') || key.startsWith('users:')) {
        this.cache.delete(key);
      }
    }
  }

  private incrementRequestCount(): void {
    this.requestCount++;
    this.pendingRequests++;
  }

  private logOperation(operation: string, details: Record<string, unknown>): void {
    try {
      this.analyticsService.trackEvent('user_management', operation, details);
    } catch {
      console.warn(`Failed to log operation: ${operation}`);
    }
  }

  private handleError<T>(error: unknown, operation: string, context: Record<string, unknown>): ApiResponse<T> {
    this.pendingRequests = Math.max(0, this.pendingRequests - 1);
    const errorMessage = error instanceof Error ? error.message : String(error);
    this.lastError = errorMessage;

    try {
      this.analyticsService.trackError('user_management', operation, errorMessage, context);
    } catch {
      console.error(`Operation ${operation} failed:`, errorMessage);
    }

    return {
      success: false,
      data: null,
      error: errorMessage,
      statusCode: 500,
      timestamp: new Date().toISOString(),
    };
  }
}

// --- Stub service interfaces (used by the main service above) ---

export interface NotificationService {
  sendNotification(userId: string, title: string, message: string): Promise<boolean>;
  sendEmailNotification(userId: string, subject: string, body: string): Promise<boolean>;
  sendSmsNotification(userId: string, message: string): Promise<boolean>;
}

export interface AnalyticsService {
  trackEvent(category: string, action: string, data?: Record<string, unknown>): void;
  trackError(category: string, action: string, errorMessage: string, context?: Record<string, unknown>): void;
  trackTiming(category: string, variable: string, durationMs: number): void;
}