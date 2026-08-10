// Plain TS class with no @angular/router import.
// Must produce zero Φroute/Φguard/Φresolver markers.

export class PlainService {
  private items: string[] = [];

  add(item: string): void {
    this.items.push(item);
  }

  getAll(): string[] {
    return this.items;
  }
}