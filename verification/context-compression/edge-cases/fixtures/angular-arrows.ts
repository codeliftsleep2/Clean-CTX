import { Component } from '@angular/core';
import { map, tap } from 'rxjs/operators';

@Component({ selector: 'app-edge', template: '' })
export class EdgeComponent {
  constructor(private repo: Repository, readonly clock: Clock) {}

  load = () => this.source.pipe(
    map(x => this.transform(x)),
    tap(x => this.audit(x)),
  ).subscribe({
    next: value => this.save(value),
    error: error => this.log(error),
  });

  cancel = () => queueMicrotask(() => this.abort());
  refresh = (...items: string[]) => this.external(...items);
}
