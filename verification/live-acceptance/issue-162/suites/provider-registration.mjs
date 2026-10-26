// Agent MaxHeadRoom: independent operator suite; maximum 500 checks.
import {runSuite} from '../suite-runner.mjs';
await runSuite({"name": "provider-registration", "family": "functional", "files": ["functional.ts", "functional-neighbor.ts"], "group": "FunctionalProviderHost"});
