// src/test_files/angular/user-card.component.ts
//
// Test fixture: a real Angular component used by the Meta-Layer
// integration tests and the compression benchmark.
//
// Phase 1 of the Angular Meta-Layer plan exercises this file to
// verify that `@Component({...})`, `@Input()`, `@Output()`, and
// the constructor-injection pattern are all detected and emitted
// as `Φcmp:` / `Φin:` / `Φout:` / `Φinjects:` markers below the
// existing compacted class entry.

import { Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
    selector: 'app-user-card',
    templateUrl: './user-card.component.html',
    styleUrls: ['./user-card.component.scss']
})
export class UserCardComponent {
    @Input() userId: string = '';
    @Input() userName: string = '';
    @Output() userDeleted = new EventEmitter<string>();

    constructor(private authService: AuthService) {}

    onDelete(): void {
        this.userDeleted.emit(this.userId);
    }

    get isAuthenticated(): boolean {
        return this.authService.isLoggedIn();
    }
}
