import { Injectable } from "@angular/core";

export interface Runner extends BaseRunner {
  run(value: string): string;
}

@Injectable()
export class Repository {
  find(value: string): string {
    return value.trim();
  }
}

@Injectable()
export class Alpha {
  private label: string = "café-λ";

  constructor(repository: Repository);
  constructor(repository: Repository, again: Repository);
  constructor(private repository: Repository, private again?: Repository) {}

  run(value: string): string {
    this.lookup(value);
    this.lookup(value);
    external(...items);
    if (value) {
      return this.repository.find(value);
    }
    return "none";
  }

  run(value: number): string {
    audit(value);
    return String(value);
  }

  unique(): string {
    return this.label;
  }

  async refresh(value: string): Promise<string> {
    const response = await fetch(value);
    console.log(response.status);
    return response.text();
  }

  observe(stream: Observable<string>): void {
    stream.subscribe(value => audit(value));
  }

  private lookup(value: string): void {
    audit(value);
  }
}

export class InheritanceProbe extends BaseService implements Runner {
  run(value: string): string {
    return value;
  }
}

export class Beta {
  run(value: string): string {
    return "beta:" + value;
  }
}

export type Identifier = string;
