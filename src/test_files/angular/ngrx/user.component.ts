import { Component, OnInit } from '@angular/core';
import { Store } from '@ngrx/store';
import { Observable } from 'rxjs';
import { loadUsers } from './user.actions';
import { selectAllUsers, selectLoading } from './user.selectors';

@Component({
  selector: 'app-user',
  template: '<div>Users</div>',
})
export class UserComponent implements OnInit {
  users$: Observable<User[]> = this.store.select(selectAllUsers);
  loading$: Observable<boolean> = this.store.select(selectLoading);

  constructor(private store: Store<AppState>) {}

  ngOnInit(): void {
    this.store.dispatch(loadUsers());
  }
}

export interface AppState {
  users: UserState;
}