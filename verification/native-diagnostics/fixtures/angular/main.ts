import { Component } from '@angular/core';
@Component({selector: 'ctx-probe', standalone: true, template: '<p>{{ title }}</p>'})
export class DiagnosticProbe { title = 'CTX_OPERATOR_ANGULAR'; }
console.log(DiagnosticProbe);
