const /*decl:top*/ target = 0;

function topLevel() {
  /*ref:top*/ use(target);
}

function parameter(/*decl:param*/ target: number) {
  /*ref:param*/ use(target);
  {
    const /*decl:nested*/ target = 1;
    /*ref:nested*/ use(target);
  }
  /*ref:param-restored*/ use(target);
}

function siblings() {
  {
    const /*decl:left*/ target = 2;
    /*ref:left*/ use(target);
  }
  {
    const /*decl:right*/ target = 3;
    /*ref:right*/ use(target);
  }
}

function first() {
  const /*decl:first*/ target = 4;
  /*ref:first*/ use(target);
}

function second() {
  const /*decl:second*/ target = 5;
  /*ref:second*/ use(target);
}

let /*decl:let*/ changing = 6;
/*ref:let*/ use(changing);

const /*decl:framework*/ selectAudit = 123;
/*ref:framework*/ use(selectAudit);
