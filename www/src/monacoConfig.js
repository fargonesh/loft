// Monarch language definition for Loft
export const loftLanguage = {
  defaultToken: '',
  tokenPostfix: '.loft',

  keywords: [
    'if', 'else', 'while', 'for', 'in', 'return', 'break', 'continue', 'match',
    'let', 'const', 'mut', 'fn', 'def', 'struct', 'enum', 'impl', 'trait', 
    'async', 'await', 'lazy', 'learn', 'teach', 'print', 'println'
  ],

  typeKeywords: [
    'num', 'str', 'bool', 'void', 'any', 'i64', 'f64', // Common types + from grammar
  ],

  literals: [
    'true', 'false', 'null'
  ],

  operators: [
    '=', '>', '<', '!', '~', '?', ':',
    '==', '<=', '>=', '!=', '&&', '||', '++', '--',
    '+', '-', '*', '/', '&', '|', '^', '%', '<<',
    '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=',
    '^=', '%=', '<<=', '>>=', '>>>='
  ],

  // Common symbols
  symbols: /[=><!~?:&|+\-*\/\^%]+/,

  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Identifiers and keywords
      [/[a-z_$][\w$]*/, { 
        cases: { 
          '@keywords': 'keyword',
          '@typeKeywords': 'type',
          '@literals': 'constant.language',
          '@default': 'identifier' 
        } 
      }],

      // Capitalized identifiers (Types or Constants)
      [/[A-Z][\w$]*/, 'type.identifier'],

      // Whitespace
      { include: '@whitespace' },

      // Delimiters and operators
      [/[{}()\[\]]/, '@brackets'],
      [/[<>](?!@symbols)/, '@brackets'],
      [/@symbols/, { 
        cases: { 
          '@operators': 'operator',
          '@default': '' 
        } 
      }],

      // Numbers
      [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
      [/0[xX][0-9a-fA-F]+/, 'number.hex'],
      [/\d+/, 'number'],

      // Strings
      [/"([^"\\]|\\.)*$/, 'string.invalid'],  // non-teminated string
      [/"/,  { token: 'string.quote', bracket: '@open', next: '@string' }],
      [/'/,  { token: 'string.quote', bracket: '@open', next: '@stringSingle' }],
      [/`/,  { token: 'string.quote', bracket: '@open', next: '@stringBacktick' }],
    ],

    comment: [
      [/[^\/*]+/, 'comment'],
      [/\/\*/,    'comment', '@push'],    // nested comment
      ["\\*/",    'comment', '@pop'],
      [/[\/*]/,   'comment']
    ],

    string: [
      [/[^\\"]+/,  'string'],
      [/@escapes/, 'string.escape'],
      [/\\./,      'string.escape.invalid'],
      [/"/,        { token: 'string.quote', bracket: '@close', next: '@pop' }]
    ],

    stringSingle: [
      [/[^\\']+/,  'string'],
      [/@escapes/, 'string.escape'],
      [/\\./,      'string.escape.invalid'],
      [/'/,        { token: 'string.quote', bracket: '@close', next: '@pop' }]
    ],

    stringBacktick: [
      [/[^\\`]+/,  'string'],
      [/@escapes/, 'string.escape'],
      [/\\./,      'string.escape.invalid'],
      [/`/,        { token: 'string.quote', bracket: '@close', next: '@pop' }]
    ],

    whitespace: [
      [/[ \t\r\n]+/, 'white'],
      [/\/\*/,       'comment', '@comment'],
      [/\/\/.*$/,    'comment'],
    ],
  },
};

export const loftTheme = {
  base: 'vs-dark', // Dark theme base
  inherit: true,
  rules: [
    { token: 'keyword', foreground: '569cd6', fontStyle: 'bold' }, // Blue keywords
    { token: 'type', foreground: '4ec9b0' }, // Teal for types
    { token: 'string', foreground: 'ce9178' }, // Orange-red strings
    { token: 'number', foreground: 'b5cea8' }, // Light green numbers
    { token: 'comment', foreground: '6a9955', fontStyle: 'italic' }, // Green comments
    { token: 'identifier', foreground: 'dcdcaa' }, // Default identifier color
    { token: 'delimiter', foreground: 'd4d4d4' },
  ],
  colors: {
    'editor.background': '#1e1e1e', // VS Code dark background
    'editor.foreground': '#d4d4d4',
  }
};
