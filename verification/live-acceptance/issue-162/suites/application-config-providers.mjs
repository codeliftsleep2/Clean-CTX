// Agent MaxHeadRoom: bounded native config membership binary acceptance.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../application-config-providers.mjs';
await runSuite({name:'application-config-providers',family:'application-config-providers',files:ownerFiles,fixture,lifecycle});
