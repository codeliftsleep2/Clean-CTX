import { Component, computed, effect, signal } from '@angular/core';
import { toSignal, toObservable } from '@angular/core/rxjs-interop';
import { of } from 'rxjs';

@Component({
  selector: 'app-user-signals',
  template: '<div>{{ fullName() }}</div>',
})
export class UserSignalsComponent {
  firstName = signal<string>('John');
  lastName = signal<string>('Doe');
  count = signal(0);

  fullName = computed(() => `${this.firstName()} ${this.lastName()}`);

  users$ = of([{ id: 1, name: 'John' }]);
  users = toSignal(this.users$, { initialValue: [] });

  count$ = toObservable(this.count);

  constructor() {
    effect(() => {
      console.log('Count changed:', this.count());
    });
  }

  increment(): void {
    this.count.update((c) => c + 1);
  }
}