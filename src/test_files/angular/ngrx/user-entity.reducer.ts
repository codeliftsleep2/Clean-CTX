import { createEntityAdapter, EntityState, createReducer, on } from '@ngrx/store';
import { loadUsersSuccess, addUser, updateUser, removeUser } from './user.actions';

export interface User {
  id: string;
  name: string;
}

export const userAdapter = createEntityAdapter<User>({
  selectId: (user) => user.id,
  sortComparer: false,
});

export interface UserEntityState extends EntityState<User> {
  loading: boolean;
}

export const initialState: UserEntityState = userAdapter.getInitialState({
  loading: false,
});

export const userEntityReducer = createReducer(
  initialState,
  on(loadUsersSuccess, (state, { users }) =>
    userAdapter.setAll(users, { ...state, loading: false })
  ),
  on(addUser, (state, { user }) => userAdapter.addOne(user, state)),
  on(updateUser, (state, { id, changes }) =>
    userAdapter.updateOne({ id, changes }, state)
  ),
  on(removeUser, (state, { id }) => userAdapter.removeOne(id, state))
);

export const { selectAll, selectEntities, selectIds, selectTotal } =
  userAdapter.getSelectors();