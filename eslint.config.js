import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

export default ts.config(
  { ignores: ["ui/dist/", "target/", "crates/**/gen/", "node_modules/"] },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: { parserOptions: { parser: ts.parser } },
  },
  {
    rules: {
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      // Keys are required for lists that reorder or change; the few static lists opt out inline.
      "svelte/require-each-key": "error",
      // We render markdown through DOMPurify (ui/src/lib/markdown.ts).
      "svelte/no-at-html-tags": "off",
    },
  },
);
