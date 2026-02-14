import stylistic from '@stylistic/eslint-plugin';
import js from '@eslint/js';
import ts from 'typescript-eslint';
import css from '@eslint/css';
import globals from "globals";

export default [
  js.configs.recommended,
  ...ts.configs.strict,
  ...ts.configs.stylistic,
  css.configs.recommended,
  {
    files: ["**/*.ts"],
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      '@stylistic/indent': ['error', 2],
      '@stylistic/semi': ['error', 'always'],
      '@typescript-eslint/no-explicit-any': ['off'],
    },
    languageOptions: {
      globals: {
        ...globals.browser,
      },
    },
  },
];
