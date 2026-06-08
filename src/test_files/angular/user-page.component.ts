// src/test_files/angular/user-page.component.ts
// Test fixture: second Angular component for cross-component bundling.

import { Component } from '@angular/core';

@Component({
    selector: 'app-user-page',
    templateUrl: './user-page.component.html',
    styleUrls: ['./user-page.component.scss']
})
export class UserPageComponent {
    pageTitle: string = 'User Management';

    onUserSelected(userId: string): void {
        console.log('Selected:', userId);
    }
}