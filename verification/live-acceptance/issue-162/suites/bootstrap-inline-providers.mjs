// Agent MaxHeadRoom: bounded inline provider occurrence acceptance.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../bootstrap-inline-providers.mjs';
await runSuite({name:'bootstrap-inline-providers',family:'bootstrap-inline-providers',files:ownerFiles,fixture,lifecycle});
