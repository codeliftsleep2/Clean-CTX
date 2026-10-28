// Agent MaxHeadRoom: bounded generation-71 production-boundary verification.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../nested-config-inputs.mjs';
await runSuite({name:'nested-config-inputs',family:'nested-config-inputs',files:ownerFiles,fixture,lifecycle});
