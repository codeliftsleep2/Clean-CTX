// src/test_files/angular/graph/user.service.ts
// Test fixture for Phase 3 cross-file graph: an @Injectable service
// that is injected by the UserCard component.

import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { LoggerService } from './logger.service';

@Injectable({ providedIn: 'root' })
export class UserService {
  constructor(private http: HttpClient, private logger: LoggerService) {}
}