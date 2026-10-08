import { TestBed } from '@angular/core/testing';
import { DiagnosticProbe } from './main';

describe('Karma Jasmine TestBed', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [DiagnosticProbe] }).compileComponents();
  });

  it('renders the Angular component with required inputs and models', () => {
    const fixture = TestBed.createComponent(DiagnosticProbe);
    fixture.componentRef.setInput('userId', 'CTX_REQUIRED_INPUT');
    fixture.componentRef.setInput('selected', true);
    fixture.detectChanges();
    expect(fixture.componentInstance.userId()).toBe('CTX_REQUIRED_INPUT');
    expect(fixture.componentInstance.selected()).toBe(true);
    fixture.componentInstance.selected.set(false);
    expect(fixture.componentInstance.selected()).toBe(false);
    let emitted = '';
    fixture.componentInstance.changed.subscribe(value => { emitted = value; });
    fixture.componentInstance.changed.emit('CTX_TYPED_OUTPUT');
    expect(emitted).toBe('CTX_TYPED_OUTPUT');
    expect(fixture.nativeElement.textContent).toContain('CTX_OPERATOR_ANGULAR');
  });

  it('uses a Jasmine spy', () => {
    const fixture = TestBed.createComponent(DiagnosticProbe);
    const callback = spyOn(fixture.componentInstance, 'renderLabel').and.returnValue('JASMINE_SPY');
    expect(fixture.componentInstance.renderLabel()).toBe('JASMINE_SPY');
    expect(callback).toHaveBeenCalledTimes(1);
  });
});
