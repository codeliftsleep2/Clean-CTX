export class PlainService {
  private data: string[] = [];

  addItem(item: string): void {
    this.data.push(item);
  }

  getItems(): string[] {
    return this.data;
  }
}