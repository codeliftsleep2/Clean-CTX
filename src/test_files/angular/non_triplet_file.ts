// src/test_files/angular/non_triplet_file.ts
// Test fixture: standalone service that should NOT be bundled.

import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class AnalyticsService {
    trackEvent(category: string, action: string): void {
        console.log(`[Analytics] ${category}: ${action}`);
    }
}