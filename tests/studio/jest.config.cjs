const path = require('path');
const studio = process.env.LIBERTAS_STUDIO;
if (!studio) throw new Error('Set LIBERTAS_STUDIO to the existing libertas/vscode checkout.');
module.exports = {
  rootDir: __dirname,
  testEnvironment: 'node',
  testMatch: ['**/*.test.js'],
  moduleDirectories: ['node_modules', path.join(studio, 'node_modules')],
  transform: {
    '^.+\\.tsx?$': [path.join(studio, 'node_modules/ts-jest'), {
      tsconfig: { module: 'CommonJS', target: 'ES2022', esModuleInterop: true, skipLibCheck: true },
      diagnostics: false,
    }],
  },
  testTimeout: 120000,
};
