// Agent MaxHeadRoom: bounded legacy call/Module binary acceptance.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../legacy-bootstrap.mjs';
await runSuite({name:'legacy-bootstrap',family:'legacy-bootstrap',files:ownerFiles,fixture,lifecycle});
