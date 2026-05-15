// Tests still author against `../src/X` imports for type-aware editing,
// but at runtime jest redirects them to `../dist/X` so each module's
// `__dirname` resolves to `dist/`. That keeps `node:worker_threads`
// happy (it cannot load `.ts` directly) without a runtime fallback in
// production code. The `pretest` script in package.json builds `dist/`
// before this config takes effect.
module.exports = {
    preset: 'ts-jest',
    testEnvironment: 'node',
    transform: { '^.+\\.ts?$': ['ts-jest', { tsconfig: 'tsconfig.test.json' }] },
    testRegex: '/test/.+\\.test\\.ts$',
    moduleNameMapper: {
        '^\\.\\./src/(.*)$': '<rootDir>/dist/$1',
    },
};
