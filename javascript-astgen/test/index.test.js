const astGenerator = require("../index.js");

test("Test docker ast generation", () => {
  const ast = astGenerator.toDockerAst("./test/Dockerfile");
  expect(ast);
});

test("Test JS ast generation", () => {
  const ast = astGenerator.toJSAst("./test/test001.js");
  expect(ast);
});
