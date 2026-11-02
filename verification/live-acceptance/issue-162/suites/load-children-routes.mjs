// Agent MaxHeadRoom: bounded routes target acceptance.
import {runSuite} from '../suite-runner.mjs';
import {scenario} from '../load-children.mjs';
await runSuite({name:'load-children-routes',family:'load-children-routes',...scenario('routes')});
