import { createAction, props } from '@ngrx/store';

export const loadUsers = createAction('[User] Load Users');
export const loadUsersSuccess = createAction(
  '[User] Load Users Success',
  props<{ users: User[] }>()
);
export const loadUsersFailure = createAction(
  '[User] Load Users Failure',
  props<{ error: string }>()
);
export const addUser = createAction(
  '[User] Add User',
  props<{ user: User }>()
);
export const updateUser = createAction(
  '[User] Update User',
  props<{ id: string; changes: Partial<User> }>()
);
export const removeUser = createAction(
  '[User] Remove User',
  props<{ id: string }>()
);

export interface User {
  id: string;
  name: string;
}