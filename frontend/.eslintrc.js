module.exports = {
  root: true,
  env: {
    browser: true,
    node: true,
  },
  extends: [
    '@nuxtjs/eslint-config-typescript',
    'plugin:nuxt/recommended',
    'google',
  ],
  plugins: [
  ],
  // add your custom rules here
  rules: {
    'vue/html-indent': 'off',
    'no-console': 'off',
    'max-len': ['error', { 'code': 120 }],
  },
}
