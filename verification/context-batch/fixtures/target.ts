export class ContextTarget {
  update(value: number): number {
    return value + 1;
  }

  untouched(value: number): number {
    return value - 1;
  }
}
