// src/test_files/angular/graph/user-page.component.ts
// Test fixture for Phase 3 cross-file graph: a parent component that
// uses the UserCard component and injects the UserService.

import { Component, OnInit } from '@angular/core';
import { UserService } from './user.service';

@Component({
  selector: 'app-user-page',
  template: '<app-user-card [userId]="selectedId"></app-user-card>'
})
export class UserPageComponent implements OnInit {
  selectedId: string = '';

  constructor(public userService: UserService) {}

  ngOnInit(): void {
    this.selectedId = '123';
  }
}