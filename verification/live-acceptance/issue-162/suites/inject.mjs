// Agent MaxHeadRoom: independent operator suite; maximum 500 checks.
import {runSuite} from '../suite-runner.mjs';
await runSuite({"name": "inject", "family": "inject", "files": ["di.ts"]});
