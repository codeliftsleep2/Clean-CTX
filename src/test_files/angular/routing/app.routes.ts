import { Routes } from '@angular/router';
import { AuthGuard } from './auth.guard';
import { UserResolver } from './user.resolver';
import { UserListComponent } from './user-list.component';
import { UserDetailComponent } from './user-detail.component';

export const appRoutes: Routes = [
  {
    path: '',
    component: UserListComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'users',
    component: UserListComponent,
    canActivate: [AuthGuard],
    resolve: { user: UserResolver },
  },
  {
    path: 'users/:id',
    loadComponent: () => import('./user-detail.component').then(m => m.UserDetailComponent),
    canActivate: [AuthGuard],
    resolve: { user: UserResolver },
  },
  {
    path: 'admin',
    loadChildren: () => import('./admin.routes').then(m => m.adminRoutes),
    canLoad: [AuthGuard],
  },
  {
    path: '**',
    redirectTo: '',
  },
];