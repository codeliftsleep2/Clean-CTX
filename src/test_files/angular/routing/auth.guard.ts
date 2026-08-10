import { Injectable } from '@angular/core';
import { CanActivate, CanLoad, Router } from '@angular/router';

@Injectable({ providedIn: 'root' })
export class AuthGuard implements CanActivate, CanLoad {
  constructor(private router: Router) {}

  canActivate(): boolean {
    return true;
  }

  canLoad(): boolean {
    return true;
  }
}

export const adminGuard: CanActivateFn = () => {
  return true;
};