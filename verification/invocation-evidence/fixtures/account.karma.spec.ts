import { TestBed } from '@angular/core/testing';
import { AccountComponent } from './account.component';
// Karma/Jasmine uses the global describe/it/expect functions.
describe('AccountComponent Karma', () => {
  beforeEach(() => TestBed.configureTestingModule({ imports: [AccountComponent], providers: [] }));
  it('renders with Jasmine', () => {
    const fixture = TestBed.createComponent(AccountComponent);
    expect(fixture.componentInstance).toBeTruthy();
  });
});
