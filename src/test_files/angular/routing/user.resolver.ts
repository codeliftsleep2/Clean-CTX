import { Injectable } from '@angular/core';
import { Resolve } from '@angular/router';
import { Observable } from 'rxjs';
import { UserService } from './user.service';

@Injectable({ providedIn: 'root' })
export class UserResolver implements Resolve<User> {
  constructor(private userService: UserService) {}

  resolve(): Observable<User> {
    return this.userService.getUser();
  }
}

export const userDetailResolver: ResolveFn<User> = (route) => {
  return route.params['id'];
};