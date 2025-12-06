// @ts-check
const eslint = require("@eslint/js");
const tseslint = require("@typescript-eslint/eslint-plugin");
const tsparser = require("@typescript-eslint/parser");
const angular = require("@angular-eslint/eslint-plugin");
const angularTemplate = require("@angular-eslint/eslint-plugin-template");
const angularTemplateParser = require("@angular-eslint/template-parser");
const prettier = require("eslint-plugin-prettier");
const prettierConfig = require("eslint-config-prettier");

module.exports = [
  {
    ignores: ["dist/**", ".angular/**", "node_modules/**", "coverage/**"],
  },
  // TypeScript files
  {
    files: ["**/*.ts"],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        project: ["./tsconfig.json", "./tsconfig.app.json", "./tsconfig.spec.json"],
        tsconfigRootDir: __dirname,
        ecmaVersion: 2022,
        sourceType: "module",
      },
      globals: {
        console: "readonly",
        window: "readonly",
        document: "readonly",
        navigator: "readonly",
        describe: "readonly",
        it: "readonly",
        expect: "readonly",
        beforeEach: "readonly",
        afterEach: "readonly",
        vi: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": tseslint,
      "@angular-eslint": angular,
      prettier: prettier,
    },
    rules: {
      ...eslint.configs.recommended.rules,
      ...tseslint.configs.recommended.rules,
      ...angular.configs.recommended.rules,
      ...prettierConfig.rules,
      "@angular-eslint/directive-selector": [
        "error",
        {
          type: "attribute",
          prefix: "app",
          style: "camelCase",
        },
      ],
      "@angular-eslint/component-selector": [
        "error",
        {
          type: "element",
          prefix: "app",
          style: "kebab-case",
        },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_" },
      ],
      "@angular-eslint/prefer-inject": "warn",
      "no-console": "warn",
      "prettier/prettier": "error",
    },
  },
  // HTML templates
  {
    files: ["**/*.html"],
    languageOptions: {
      parser: angularTemplateParser,
    },
    plugins: {
      "@angular-eslint/template": angularTemplate,
      prettier: prettier,
    },
    rules: {
      ...angularTemplate.configs.recommended.rules,
      ...angularTemplate.configs.accessibility.rules,
      ...prettierConfig.rules,
      "@angular-eslint/template/prefer-control-flow": "warn",
      "@angular-eslint/template/prefer-self-closing-tags": "error",
      "prettier/prettier": "error",
    },
  },
];
