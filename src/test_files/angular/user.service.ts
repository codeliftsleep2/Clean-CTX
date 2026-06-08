// src/test_files/angular/user.service.ts
//
// Test fixture: a real Angular @Injectable service used by the
// Meta-Layer integration tests.
//
// Phase 1 of the Angular Meta-Layer plan exercises this file to
// verify that `@Injectable({providedIn: 'root'})` produces a
// `Φsvc:UserService scope=root` marker line, and that constructor
// parameters with `private` access modifiers produce
// `Φinjects:[<Type>]` markers.

import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';

@Injectable({ providedIn: 'root' })
export class UserService {
    constructor(private http: HttpClient, private auth: AuthService) {}

    getUser(id: string): Promise<unknown> {
        return this.http.get(`/api/users/${id}`).toPromise();
    }

    isAuthenticated(): boolean {
        return this.auth.isLoggedIn();
    }
}
