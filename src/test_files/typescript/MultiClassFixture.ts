// src/test_files/typescript/MultiClassFixture.ts
//
// Multi-class TypeScript regression fixture.
//
// Tests the per-class metadata invariant:
//   A meta-layer may inspect only the exact source span belonging to the
//   type it is enriching. It must never infer ownership from neighboring
//   or whole-file text.
//
// This file contains multiple classes with different Angular decorators
// and plain classes interspersed. Each class's markers must be emitted
// ONLY for that class, never for another class.

import { Component, Injectable, Input, Output, EventEmitter } from '@angular/core';

// ════════════════════════════════════════════════════════════════════════
// Class 1: @Component with selector 'app-hello'
// Expected: Φcmp:HelloComponent sel=app-hello
// ════════════════════════════════════════════════════════════════════════
@Component({
    selector: 'app-hello',
    template: '<h1>Hello</h1>'
})
export class HelloComponent {
    @Input() name: string = '';
}

// ════════════════════════════════════════════════════════════════════════
// Class 2: Plain class — NO Angular decorator
// Expected: NO Φ markers of any kind
// ════════════════════════════════════════════════════════════════════════
export class DataModel {
    constructor(public id: number, public label: string) {}
}

// ════════════════════════════════════════════════════════════════════════
// Class 3: @Injectable service
// Expected: Φsvc:DataService scope=root
// CRITICAL: Must NOT inherit HelloComponent's Φcmp marker
// ════════════════════════════════════════════════════════════════════════
@Injectable({ providedIn: 'root' })
export class DataService {
    fetchAll(): Promise<DataModel[]> { return Promise.resolve([]); }
}

// ════════════════════════════════════════════════════════════════════════
// Class 4: @Component with DIFFERENT selector 'app-goodbye'
// Expected: Φcmp:GoodbyeComponent sel=app-goodbye
// CRITICAL: Must NOT inherit HelloComponent's 'app-hello' or DataService's Φsvc
// ════════════════════════════════════════════════════════════════════════
@Component({
    selector: 'app-goodbye',
    template: '<h1>Goodbye</h1>'
})
export class GoodbyeComponent {
    @Output() closed = new EventEmitter<void>();
}

// ════════════════════════════════════════════════════════════════════════
// Class 5: Plain interface — NO Angular decorator
// Expected: NO Φ markers
// ════════════════════════════════════════════════════════════════════════
export interface ApiResponse {
    success: boolean;
    data: unknown;
}
