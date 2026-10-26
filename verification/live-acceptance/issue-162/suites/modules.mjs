// Agent MaxHeadRoom: independent operator suite; maximum 500 checks.
import {runSuite} from '../suite-runner.mjs';
await runSuite({"name": "modules", "family": "modules", "files": ["app.ts", "dashboard.ts"]});
