// Reusable synthetic fixture definitions. Generated source belongs under target/.
const frameworkCases = [
  ['Controller', 'Microsoft.AspNetCore.Mvc.ControllerBase'],
  ['Hub', 'Microsoft.AspNetCore.SignalR.Hub'],
  ['MapperProfile', 'AutoMapper.Profile'],
  ['DbContext', 'Microsoft.EntityFrameworkCore.DbContext'],
];
const genericMethods = [
  'protected void Nf<T>() {}',
  'protected void Nf<T, U>(int value) {}',
  'protected void Other<A, B, C>() {}',
].join('\n');

function payload(role, name) {
  switch (role) {
    case 'Controller': return '[global::Microsoft.AspNetCore.Mvc.HttpGet] public void Run() {}';
    case 'Hub': return 'public void Run() {}';
    case 'MapperProfile': return `public ${name}() { CreateMap<Source, Destination>(); }`;
    case 'DbContext': return 'public DbSet<IdentityRow> Rows { get; set; }';
    default: return '';
  }
}

function expectation(role, name, file, targets) {
  const relations = {
    Controller: 'ControllerAction', Hub: 'HubMethodTargets',
    MapperProfile: 'MapsTo', DbContext: 'HasEntity', TestClass: 'Defines',
  };
  const types = {
    Controller: 'Action', Hub: 'HubMethod', MapperProfile: 'Entity',
    DbContext: 'Entity', TestClass: 'TestClass',
  };
  return { role, name, file, relation: relations[role], targetType: types[role], targets };
}

export function makeFixtures() {
  const files = {};
  const cases = [];
  const actionNames = ['ActionA', 'ActionB', 'ActionC', 'ActionD', 'ActionE'];
  const attributes = ['HttpGet', 'HttpPost', 'HttpPut', 'HttpDelete', 'HttpPatch'];
  let editCase;
  for (const reverse of [false, true]) {
    for (const position of [0, 2, 5]) {
      for (const nonempty of [false, true]) {
        const name = `Endpoint${reverse ? 'Reverse' : 'Forward'}${position}${nonempty ? 'Body' : 'Empty'}`;
        const members = actionNames.map((action, index) =>
          `[${attributes[index]}] public void ${action}() {}`);
        if (reverse) members.reverse();
        members.splice(position, 0, nonempty
          ? `public ${name}(int value) { var local = value; }`
          : `public ${name}() {}`);
        const file = `${name}.cs`;
        files[file] = `using Microsoft.AspNetCore.Mvc; public class ${name} : ControllerBase {\n${members.join('\n')}\n}\n`;
        const item = expectation('Controller', name, file, actionNames);
        cases.push(item);
        editCase ??= item;
      }
    }
  }
  files['Overloads.cs'] = `using Microsoft.AspNetCore.Mvc;
public class OverloadedEndpoint : ControllerBase {
    public OverloadedEndpoint() {}
    [HttpGet] public void Read() {}
    [HttpPost] public void Read(int value) {}
    [HttpDelete] public void Last() {}
}
`;
  // MCP action identity is name-based here: distinct overload attribute ownership
  // is proved by the Rust regression, not inferred from edge multiplicity.
  cases.push(expectation('Controller', 'OverloadedEndpoint', 'Overloads.cs', ['Read', 'Last']));

  const lifecycle = [];
  for (const [role, framework] of frameworkCases) {
    for (const variant of ['Methods', 'PartialBase', 'PartialOther', 'Generic1', 'Generic2', 'Generic3']) {
      const prefix = `${role}${variant}`;
      const rootName = `${prefix}Root`;
      const middleName = `${prefix}Middle`;
      const leafName = `${prefix}Leaf`;
      const arity = variant.startsWith('Generic') ? Number(variant.at(-1)) : 0;
      const parameters = ['T', 'U', 'V'].slice(0, arity).join(', ');
      const suffix = arity ? `<${parameters}>` : '';
      const partial = variant.startsWith('Partial');
      const baseFile = `${prefix}-base.cs`;
      const otherFile = `${prefix}-part.cs`;
      const leafFile = `${prefix}-descendants.cs`;
      const ownMethods = arity ? 'protected void M<A,B,C,D>() {}' : genericMethods;
      const header = `public ${partial ? 'partial ' : ''}class ${rootName}${suffix}`;
      files[baseFile] = `namespace Bases { ${header} : global::${framework} {\n${variant === 'PartialOther' ? '' : ownMethods}\n} }\n`;
      if (partial) {
        files[otherFile] = `namespace Bases { public partial class ${rootName} {\n${variant === 'PartialOther' ? ownMethods : ''}\n} }\n`;
      }
      // Class arity is exercised at the base declaration; closed descendants
      // retain ordinary public names on the MCP role surfaces.
      const arguments_ = arity ? `<${['int', 'string', 'long'].slice(0, arity).join(', ')}>` : '';
      files[leafFile] = `${role === 'DbContext' ? 'using Microsoft.EntityFrameworkCore;\n' : ''}namespace Consumers {
public class ${middleName} : global::Bases.${rootName}${arguments_} {}
public class ${leafName} : ${middleName} { ${payload(role, leafName)} }
}\n`;
      const targets = role === 'MapperProfile' ? ['Destination']
        : role === 'DbContext' ? ['IdentityRow'] : ['Run'];
      const item = expectation(role, leafName, leafFile, targets);
      cases.push(item);
      if (variant === 'PartialBase') {
        lifecycle.push({
          item, file: baseFile,
          replacement: `namespace Bases { ${header} { ${genericMethods} } }\n`,
        });
      }
    }
  }
  files['Tests.cs'] = `[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClass]
public class IdentityTests { public IdentityTests() {} public void M<A,B,C,D>() {} }
`;
  // Self-Defines is an occurrence-registration carrier, not graph adjacency.
  cases.push(expectation('TestClass', 'IdentityTests', 'Tests.cs', []));
  files['Models.cs'] = 'public class IdentityRow {} public class Source {} public class Destination {}\n';
  files['Lookalike.cs'] = `namespace Local { public class ControllerBase {} }
public class LookalikeEndpoint : Local.ControllerBase {
    public LookalikeEndpoint() {} public void M<T>() {} public void Run() {}
}
`;
  return { files, cases, lifecycle, editCase, negative: 'LookalikeEndpoint' };
}
