import { Component, input, model, output } from '@angular/core';
@Component({selector: 'ctx-probe', standalone: true, template: '<p>{{ title }}</p>'})
export class DiagnosticProbe {
  readonly userId = input.required < string > ();
  readonly selected = model.required < boolean > ();
  readonly changed = output < string > ();
  title = 'CTX_OPERATOR_ANGULAR';
  renderLabel(): string { return this.title; }
}
console.log(DiagnosticProbe);
