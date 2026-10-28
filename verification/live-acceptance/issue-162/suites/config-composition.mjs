// Agent MaxHeadRoom: bounded named config composition acceptance.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../config-composition.mjs';
await runSuite({name:'config-composition',family:'config-composition',files:ownerFiles,fixture,lifecycle});
