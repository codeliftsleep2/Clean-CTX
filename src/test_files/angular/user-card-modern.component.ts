import { Component, input, output, model, inject } from '@angular/core';
import { UserService } from './user.service';

@Component({
  selector: 'app-user-card-modern',
  template: './user-card-modern.component.html',
  standalone: true,
  imports: [],
})
export class UserCardModernComponent {
  // Signal-based inputs (Angular 17.1+)
  readonly userId = input.required < string > ();
  readonly userName = input<string>('default');
  readonly items = input < any[] > ([]);

  // Model (two-way binding signal) (Angular 17.1+)
  readonly selected = model.required<boolean>();

  // Signal-based output (Angular 17.1+)
  readonly userDeleted = output<string>();

  // viewChild / contentChild signals
  // private readonly el = viewChild<ElementRef>('someRef');

  // Inject function usage (instead of constructor DI)
  private readonly userService = inject(UserService);

  // Constructor-based DI (legacy style)
  // constructor(private logger: LoggerService) {}

  toggleSelected(): void {
    this.selected.update((prev) => !prev);
  }

  onDelete(): void {
    this.userDeleted.emit('deleted');
  }
}