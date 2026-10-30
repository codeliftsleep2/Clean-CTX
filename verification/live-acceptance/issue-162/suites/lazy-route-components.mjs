// Agent MaxHeadRoom: bounded canonical lazy binary acceptance.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../lazy-route-components.mjs';
await runSuite({name:'lazy-route-components',family:'lazy-route-components',files:ownerFiles,fixture,lifecycle});
