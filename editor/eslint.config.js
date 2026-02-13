import stylistic from '@stylistic/eslint-plugin';
import globals from "globals";
import js from '@eslint/js';

export default [
  js.configs.recommended,
  {
    files: ["**/*.ts", "**/*.js"],
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      '@stylistic/indent': ['error', 2],
      '@stylistic/semi': ['error', 'always'],
    },
    languageOptions: {
      globals: {
        ...globals.browser,
      },
    },
  }
];
