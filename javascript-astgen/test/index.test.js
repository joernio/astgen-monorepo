const astGenerator = require("../index.js");

test("Test docker ast generation", () => {
  const ast = astGenerator.toDockerAst("./test/Dockerfile");
  expect(ast);
});

test("Test docker ast from content", () => {
  const ast = astGenerator.toDockerAst(
    null,
    `
	FROM centos:7 as builder
FROM centos:7 as new_image
COPY --from=builder /tmp /tmp
`
  );
  expect(ast);
  expect(ast.program.body[ast.program.body.length - 1].flags);
});

test("Test JS ast generation", () => {
  const ast = astGenerator.toJSAst("./test/test001.js");
  expect(ast);
});

test("Test Bash ast generation", () => {
  const ast = astGenerator.toBashAst("./test/test001.sh", null);
  expect(ast);
});

test("Test vue ast generation", () => {
  let ast = astGenerator.toVueAst("./test/App.vue");
  expect(ast);
  ast = astGenerator.toVueAst("./test/HelloWorld.vue");
  expect(ast);
});

test("Test svelte ast generation", () => {
  let ast = astGenerator.toSvelteAst("./test/App.svelte");
  expect(ast);
});
