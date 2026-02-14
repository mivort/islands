import stylistic from '@stylistic/eslint-plugin';
import js from '@eslint/js';
import css from '@eslint/css';
import globals from "globals";

export default [
  {
    files: ["**/*.ts", "**/*.js"],
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      ...js.configs.recommended.rules,
      '@stylistic/indent': ['error', 2],
      '@stylistic/semi': ['error', 'always'],
    },
    languageOptions: {
      globals: {
        ...globals.browser,
      },
    },
  },
  {
    files: ["**/*.css"],
    language: "css/css",
    plugins: { css },
    rules: { ...css.configs.recommended.rules },
  },
];
