// Agent MaxHeadRoom: bounded module target acceptance.
import {runSuite} from '../suite-runner.mjs';
import {scenario} from '../load-children.mjs';
await runSuite({name:'load-children-modules',family:'load-children-modules',...scenario('module')});
