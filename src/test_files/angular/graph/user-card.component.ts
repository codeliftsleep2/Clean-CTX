// src/test_files/angular/graph/user-card.component.ts
// Test fixture for Phase 3 cross-file graph: a component that injects
// a service and is used by a parent component.

import { Component, Input, Output, EventEmitter } from '@angular/core';
import { UserService } from './user.service';

@Component({
  selector: 'app-user-card',
  templateUrl: './user-card.component.html',
  styleUrls: ['./user-card.component.scss']
})
export class UserCardComponent {
  @Input() userId!: string;
  @Output() userSelected = new EventEmitter<string>();

  constructor(private userService: UserService) {}
}