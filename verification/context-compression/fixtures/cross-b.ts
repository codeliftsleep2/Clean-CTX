import { SharedName as ImportedShared } from "./cross-a";

export class SharedName {
  call(): string {
    return externalB();
  }
}

export class Consumer {
  constructor(private dependency: ImportedShared) {}
}
