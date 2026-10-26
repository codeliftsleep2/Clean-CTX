// Agent MaxHeadRoom: consolidated C2/C3 operator acceptance; maximum 500 checks.
import {runSuite} from '../suite-runner.mjs';
import {fixture, lifecycle, ownerFiles} from '../provider-imported-owners.mjs';
await runSuite({name:'provider-imported-owners', family:'provider-imported-owners',
  files:ownerFiles, fixture, lifecycle});
