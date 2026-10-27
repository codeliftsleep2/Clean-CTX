// Agent MaxHeadRoom: bounded bootstrap invocation binary acceptance.
import {runSuite} from '../suite-runner.mjs';
import {fixture,lifecycle,ownerFiles} from '../bootstrap-references.mjs';
await runSuite({name:'bootstrap-references',family:'bootstrap-references',files:ownerFiles,fixture,lifecycle});
