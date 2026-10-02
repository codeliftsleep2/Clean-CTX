import { Injectable } from "@angular/core";

@Injectable()
export class Repository {}

@Injectable()
export class Alpha {
  constructor(private repository: Repository) {}
}

export class BaseWorker {}

export interface RootContract {}

export interface WorkerContract extends RootContract {}

export class ConcreteWorker extends BaseWorker implements WorkerContract {}
