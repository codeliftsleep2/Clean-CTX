import { Injectable } from "@angular/core";

@Injectable()
export class Repository {}

@Injectable()
export class Alpha {
  constructor(private repository: Repository) {}
}
