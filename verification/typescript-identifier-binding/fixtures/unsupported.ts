const /*decl:target*/ target = 1;
var legacy = 2;
import { remote } from "./remote";

/*ref:var*/ use(legacy);
/*ref:import*/ use(remote);
/*ref:string*/ use("target");
/*ref:member*/ use(object.target);
/*ref:compound*/ use(wrapper(target));
type Alias = /*marked:type*/ target;
const /*marked:declaration*/ declared = 3;
