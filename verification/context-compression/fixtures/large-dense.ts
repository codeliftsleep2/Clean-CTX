import { Injectable } from "@angular/core";

export interface DensePort extends RootPort {
  execute(value: string): string;
}

@Injectable()
export class Node01 { step(value: string): string { return value + "01"; } }
@Injectable()
export class Node02 { step(value: string): string { return value + "02"; } }
@Injectable()
export class Node03 { step(value: string): string { return value + "03"; } }
@Injectable()
export class Node04 { step(value: string): string { return value + "04"; } }
@Injectable()
export class Node05 { step(value: string): string { return value + "05"; } }
@Injectable()
export class Node06 { step(value: string): string { return value + "06"; } }
@Injectable()
export class Node07 { step(value: string): string { return value + "07"; } }
@Injectable()
export class Node08 { step(value: string): string { return value + "08"; } }
@Injectable()
export class Node09 { step(value: string): string { return value + "09"; } }
@Injectable()
export class Node10 { step(value: string): string { return value + "10"; } }
@Injectable()
export class Node11 { step(value: string): string { return value + "11"; } }
@Injectable()
export class Node12 { step(value: string): string { return value + "12"; } }

@Injectable()
export class DenseCoordinator extends DenseBase implements DensePort {
  private title: string = "graphe-densé-東京";

  constructor(
    private n01: Node01,
    private n02: Node02,
    private n03: Node03,
    private n04: Node04,
    private n05: Node05,
    private n06: Node06,
    private n07: Node07,
    private n08: Node08,
    private n09: Node09,
    private n10: Node10,
    private n11: Node11,
    private n12: Node12,
  ) {}

  execute(value: string): string {
    const a = this.n01.step(value);
    const b = this.n02.step(a);
    const c = this.n03.step(b);
    const d = this.n04.step(c);
    this.n05.step(a);
    this.n06.step(a);
    this.n07.step(b);
    this.n08.step(b);
    this.n09.step(c);
    this.n10.step(c);
    this.n11.step(d);
    this.n12.step(d);
    audit(a);
    audit(a);
    external(...values);
    if (d) {
      return this.n12.step(d);
    }
    return this.title;
  }

  execute(value: number): string {
    return this.execute(String(value));
  }

  async synchronize(url: string): Promise<void> {
    const response = await fetch(url);
    console.log(response.status);
  }
}

export type DenseIdentifier = string;
