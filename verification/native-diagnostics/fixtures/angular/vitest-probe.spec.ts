import { beforeEach, describe, expect, it, vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { DiagnosticProbe } from './main';

describe('Vitest TestBed', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [DiagnosticProbe] }).compileComponents();
  });

  it('renders the Angular component', () => {
    const fixture = TestBed.createComponent(DiagnosticProbe);
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('CTX_OPERATOR_ANGULAR');
  });

  it('uses a Vitest spy', () => {
    const fixture = TestBed.createComponent(DiagnosticProbe);
    const callback = vi.spyOn(fixture.componentInstance, 'renderLabel').mockReturnValue('VITEST_SPY');
    expect(fixture.componentInstance.renderLabel()).toBe('VITEST_SPY');
    expect(callback).toHaveBeenCalledTimes(1);
  });
});
