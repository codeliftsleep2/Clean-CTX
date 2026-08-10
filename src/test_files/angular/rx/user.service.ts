import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject, Subject, of } from 'rxjs';
import { switchMap, map, catchError, shareReplay, tap } from 'rxjs/operators';

@Injectable({ providedIn: 'root' })
export class UserService {
  private apiUrl = '/api/users';

  users$: Observable<User[]> = this.http.get<User[]>('/api/users');

  refreshTrigger = new Subject<void>();

  selectedUser$ = new BehaviorSubject<User | null>(null);

  private loadUsers$ = this.refreshTrigger.pipe(
    switchMap(() => this.http.get<User[]>('/api/users')),
    map(users => users.sort((a, b) => a.name.localeCompare(b.name))),
    tap(users => console.log('Loaded users:', users.length)),
    catchError(err => of([])),
    shareReplay(1)
  );

  constructor(private http: HttpClient) {}

  getUser(id: string): Observable<User> {
    return this.http.get<User>(`/api/users/${id}`);
  }
}

export interface User {
  id: string;
  name: string;
}