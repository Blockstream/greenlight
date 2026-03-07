module.exports = {
  preset: 'ts-jest/presets/default-esm',
  testEnvironment: 'node',
  maxWorkers: 1,
  testTimeout: 120_000,
  runner: 'jest-runner',
  resetModules: true,
  restoreMocks: true,
  clearMocks: true,
  extensionsToTreatAsEsm: ['.ts'],
  transform: {
    '^.+\\.ts$': [
      'ts-jest',
      { useESM: true },
    ],
  },
  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.js$': '$1',
  },
  testMatch: ["<rootDir>/tests/**/*.spec.ts"],
  globalSetup: '<rootDir>/tests/jest.globalSetup.ts',
  globalTeardown: '<rootDir>/tests/jest.globalTeardown.ts',
};
