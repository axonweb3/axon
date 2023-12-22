module.exports = {
  env: {
    node: true,
    es2022: true,
    jest: true,
    browser: true
  },
  globals: {
    browser: 'writable',
    metamask: 'writable',
    page: 'writable'
  },
  extends: ['eslint:recommended', 'airbnb-base', 'plugin:sonarjs/recommended'],
  plugins: ['sonarjs'],
  parserOptions: {
    sourceType: 'module'
  },
  rules: {
    indent: [
      'error',
      2
    ],
    'linebreak-style': [
      'error',
      'unix'
    ],
    quotes: [
      'error',
      'double'
    ],
    semi: [
      'error',
      'always'
    ]
  }
}
