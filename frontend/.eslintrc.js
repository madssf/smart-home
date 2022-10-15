module.exports = {
  parser: "@typescript-eslint/parser",
  extends: ["@remix-run/eslint-config", "@remix-run/eslint-config/node", "plugin:@typescript-eslint/recommended"],
  plugins: ["react", "react-hooks"],
  rules: {
    "semi": ["error", "always", { "omitLastInOneLineBlock": true}],
    "comma-dangle": ["error", "always-multiline"],
    "max-len": ["error", {
      "code": 150,
      "ignoreUrls": true,
    }],
  },
};
