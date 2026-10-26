// Agent MaxHeadRoom: bounded route membership binary acceptance, max 500 checks.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../route-providers.mjs';
await runSuite({name:'route-providers',family:'route-providers',files:ownerFiles,fixture,lifecycle});
