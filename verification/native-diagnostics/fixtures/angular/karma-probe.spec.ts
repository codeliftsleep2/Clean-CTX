import { TestBed } from '@angular/core/testing';
import { DiagnosticProbe } from './main';

describe('Karma Jasmine TestBed', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [DiagnosticProbe] }).compileComponents();
  });

  it('renders the Angular component', () => {
    const fixture = TestBed.createComponent(DiagnosticProbe);
    fixture.detectChanges();
    expect(fixture.nativeElement.textContent).toContain('CTX_OPERATOR_ANGULAR');
  });

  it('uses a Jasmine spy', () => {
    const fixture = TestBed.createComponent(DiagnosticProbe);
    const callback = spyOn(fixture.componentInstance, 'renderLabel').and.returnValue('JASMINE_SPY');
    expect(fixture.componentInstance.renderLabel()).toBe('JASMINE_SPY');
    expect(callback).toHaveBeenCalledTimes(1);
  });
});
