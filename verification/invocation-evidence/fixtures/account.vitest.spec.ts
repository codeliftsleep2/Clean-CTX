import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { AccountComponent } from './account.component';
describe('AccountComponent Vitest', () => {
  const callback = vi.fn();
  beforeEach(() => TestBed.configureTestingModule({ imports: [AccountComponent], providers: [] }));
  it('renders with Vitest', () => {
    const fixture = TestBed.createComponent(AccountComponent);
    expect(fixture.componentInstance).toBeTruthy();
    callback();
  });
});
