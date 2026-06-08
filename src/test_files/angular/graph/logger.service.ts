// src/test_files/angular/graph/logger.service.ts
// Test fixture for Phase 3 cross-file graph: a simple @Injectable service
// that has no dependencies (leaf node in DI graph).

import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class LoggerService {
  log(message: string): void {
    console.log(message);
  }
}