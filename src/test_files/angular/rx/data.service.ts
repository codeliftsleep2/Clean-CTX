import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, combineLatest, forkJoin, of } from 'rxjs';
import { debounceTime, distinctUntilChanged, map, switchMap } from 'rxjs/operators';

@Injectable({ providedIn: 'root' })
export class DataService {
  searchTerm$ = new BehaviorSubject<string>('');

  results$ = this.searchTerm$.pipe(
    debounceTime(300),
    distinctUntilChanged(),
    switchMap(term => this.search(term))
  );

  combined$ = combineLatest([this.searchTerm$, this.results$]);

  allData$ = forkJoin([this.loadUsers(), this.loadOrders()]);

  constructor(private http: HttpClient) {}

  private search(term: string): Observable<string[]> {
    return this.http.get<string[]>(`/api/search?q=${term}`);
  }

  private loadUsers(): Observable<string[]> {
    return of(['user1', 'user2']);
  }

  private loadOrders(): Observable<string[]> {
    return of(['order1', 'order2']);
  }
}