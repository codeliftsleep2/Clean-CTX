import { map, switchMap, tap } from 'rxjs/operators';
const decoy = '.pipe(map(fake), tap(fake))';
// source.pipe(map(fake), tap(fake));
export class Reactive {
  stream$ = this.source . pipe /* written boundary */ (
    map(value => ({ value, pair: [1, 2] })),
    switchMap(value => this.api.load(value, { retry: [1, 2] })),
    tap(log)
  );
  unicode$ = this.source.pipe(
    map(x => "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaélong-text"),
    tap(log)
  );
  run() {
    service.Save<User>(item, ...items);
    Lookup(item);
    Lookup(item);
    handlers[name](item);
  }
  getOther() { return this.source.pipe(map(x => x), tap(log)); }
}
