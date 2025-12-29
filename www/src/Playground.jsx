import { useState, useEffect, useRef } from 'react';
import Editor from '@monaco-editor/react';
import { 
  BrutalCard, 
  Button, 
  Heading, 
  Text,
} from 'botanical-ui';
import init, { run_loft, format_loft, init_panic_hook, get_stdlib_metadata } from './pkg/loft';

const LOFT_MONARCH = {
  defaultToken: '',
  tokenPostfix: '.lf',
  keywords: [
    'if', 'else', 'while', 'for', 'in', 'return', 'break', 'continue', 'match',
    'let', 'const', 'mut', 'fn', 'def', 'struct', 'enum', 'impl', 'trait',
    'async', 'await', 'learn', 'teach'
  ],
  typeKeywords: [
    'int', 'float', 'str', 'bool', 'void', 'any', 'Self'
  ],
  operators: [
    '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
    '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
    '<<', '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
    '%=', '<<=', '>>=', '>>>='
  ],
  symbols: /[=><!~?:&|+\-*\/\^%]+/,
  tokenizer: {
    root: [
      [/[a-z_$][\w$]*/, {
        cases: {
          '@typeKeywords': 'keyword',
          '@keywords': 'keyword',
          '@default': 'identifier'
        }
      }],
      [/[A-Z][\w$]*/, 'type.identifier'],
      { include: '@whitespace' },
      [/[{}()\[\]]/, '@brackets'],
      [/@symbols/, {
        cases: {
          '@operators': 'operator',
          '@default': ''
        }
      }],
      [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
      [/0[xX][0-9a-fA-F]+/, 'number.hex'],
      [/\d+/, 'number'],
      [/[;,.]/, 'delimiter'],
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
      [/'[^\\']'/, 'string'],
      [/'/, 'string.invalid'],
      [/`/, { token: 'string.quote', bracket: '@open', next: '@templateString' }],
    ],
    string: [
      [/[^\\"]+/, 'string'],
      [/\\./, 'string.escape'],
      [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
    ],
    templateString: [
      [/[^\\`$]+/, 'string'],
      [/\\./, 'string.escape'],
      [/\$\{/, { token: 'delimiter.bracket', next: '@bracketCounting' }],
      [/`/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
    ],
    bracketCounting: [
      [/\{/, { token: 'delimiter.bracket', next: '@bracketCounting' }],
      [/\}/, { token: 'delimiter.bracket', next: '@pop' }],
      { include: 'root' },
    ],
    whitespace: [
      [/[ \t\r\n]+/, 'white'],
      [/\/\*/, 'comment', '@comment'],
      [/\/\/.*$/, 'comment'],
    ],
    comment: [
      [/[^\/*]+/, 'comment'],
      [/\/\*/, 'comment', '@push'],
      [/\*\//, 'comment', '@pop'],
      [/[\/*]/, 'comment']
    ],
  },
};

const DEFAULT_CODE = `// Welcome to the Loft Playground!

fn main() {
    let message = "Hello, Loft!";
    print(message);
    
    return 42;
}

main();`;

const Playground = () => {
  const [code, setCode] = useState(() => {
    const saved = localStorage.getItem('loft_playground_code');
    return saved || DEFAULT_CODE;
  });
  const [output, setOutput] = useState('');
  const [error, setError] = useState('');
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [stdlib, setStdlib] = useState({ builtins: [], globals: [] });
  const monacoRef = useRef(null);

  useEffect(() => {
    localStorage.setItem('loft_playground_code', code);
  }, [code]);

  useEffect(() => {
    init().then(() => {
      init_panic_hook();
      try {
        const metadata = JSON.parse(get_stdlib_metadata());
        setStdlib(metadata);
      } catch (e) {
        console.error("Failed to load stdlib metadata", e);
      }
      setWasmLoaded(true);
    });
  }, []);

  const handleRun = () => {
    if (!wasmLoaded) return;
    try {
      const result = run_loft(code);
      setOutput(result.output);
      setError(result.error || '');
    } catch (e) {
      setError(e.toString());
    }
  };

  const handleFormat = () => {
    if (!wasmLoaded) return;
    const formatted = format_loft(code);
    setCode(formatted);
  };

  const handleClear = () => {
    setOutput('');
    setError('');
  };

  const handleEditorDidMount = (editor, monaco) => {
    monacoRef.current = monaco;
    
    // Register Loft language
    monaco.languages.register({ id: 'loft' });
    monaco.languages.setMonarchTokensProvider('loft', LOFT_MONARCH);
    
    // Autocomplete
    monaco.languages.registerCompletionItemProvider('loft', {
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn,
        };
        
        const suggestions = [
          ...LOFT_MONARCH.keywords.map(k => ({
            label: k,
            kind: monaco.languages.CompletionItemKind.Keyword,
            insertText: k,
            range,
          })),
          ...LOFT_MONARCH.typeKeywords.map(k => ({
            label: k,
            kind: monaco.languages.CompletionItemKind.TypeParameter,
            insertText: k,
            range,
          })),
          ...stdlib.globals.map(g => ({
            label: g,
            kind: monaco.languages.CompletionItemKind.Function,
            insertText: g + '($0)',
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
          })),
          ...stdlib.builtins.flatMap(b => [
            {
              label: b.name,
              kind: monaco.languages.CompletionItemKind.Module,
              insertText: b.name,
              range,
            },
            ...b.methods.map(m => ({
              label: `${b.name}.${m}`,
              kind: monaco.languages.CompletionItemKind.Method,
              insertText: `${b.name}.${m}($0)`,
              insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
              range,
            }))
          ]),
          {
            label: 'main',
            kind: monaco.languages.CompletionItemKind.Snippet,
            insertText: `fn main() {
	$0
}`,
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
          }
        ];
        
        return { suggestions };
      }
    });
  };

  return (
    <div className="flex flex-col h-[calc(100vh-64px)] bg-bio-cream">
      {/* Toolbar */}
      <div className="flex justify-between items-center px-6 py-4 border-b-4 border-black bg-white">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 bg-bio-green border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] flex items-center justify-center text-white font-bold text-xl">
            l
          </div>
          <Heading level={2} className="m-0">Playground</Heading>
        </div>
        <div className="flex gap-3">
          <Button 
            onClick={handleClear} 
            className="bg-white text-black border-2 border-black px-4 py-2 hover:bg-gray-100 transition-colors shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:shadow-none active:translate-x-[2px] active:translate-y-[2px]"
          >
            Clear Output
          </Button>
          <Button 
            onClick={handleFormat} 
            disabled={!wasmLoaded}
            className="bg-white text-black border-2 border-black px-4 py-2 hover:bg-gray-100 transition-colors shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:shadow-none active:translate-x-[2px] active:translate-y-[2px]"
          >
            Format
          </Button>
          <Button 
            onClick={handleRun} 
            disabled={!wasmLoaded} 
            className="bg-bio-green text-white border-2 border-black px-6 py-2 hover:opacity-90 transition-colors shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:shadow-none active:translate-x-[2px] active:translate-y-[2px] font-bold"
          >
            {wasmLoaded ? 'Run Code' : 'Loading...'}
          </Button>
        </div>
      </div>

      <div className="flex flex-1 min-h-0 p-6 gap-6">
        {/* Editor Section */}
        <div className="flex-1 flex flex-col min-h-0">
          <div className="flex-1 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] bg-white overflow-hidden flex flex-col">
            <div className="bg-black text-white px-4 py-1 text-xs font-mono flex justify-between items-center">
              <span>main.lf</span>
              <span className="opacity-50">Loft v0.1.0</span>
            </div>
            <div className="flex-1 relative">
              <Editor
                height="100%"
                defaultLanguage="loft"
                value={code}
                onChange={(v) => setCode(v)}
                onMount={handleEditorDidMount}
                theme="vs-dark"
                options={{
                  fontSize: 16,
                  minimap: { enabled: false },
                  scrollBeyondLastLine: false,
                  automaticLayout: true,
                  padding: { top: 16, bottom: 16 },
                  fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                  lineNumbers: 'on',
                  renderLineHighlight: 'all',
                  scrollbar: {
                    vertical: 'visible',
                    horizontal: 'visible',
                    useShadows: false,
                    verticalScrollbarSize: 10,
                    horizontalScrollbarSize: 10,
                  }
                }}
              />
            </div>
          </div>
        </div>

        {/* Output Section */}
        <div className="w-1/3 flex flex-col gap-6 min-h-0">
          <div className="flex-1 flex flex-col min-h-0 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] bg-white">
            <div className="bg-black text-white px-4 py-1 text-xs font-mono">
              OUTPUT
            </div>
            <div className="flex-1 bg-[#1e1e1e] text-green-400 p-4 font-mono overflow-auto whitespace-pre-wrap text-sm selection:bg-green-900 selection:text-white">
              {output || (wasmLoaded ? "// Output will appear here..." : "// Initializing runtime...")}
            </div>
          </div>

          {error && (
            <div className="h-1/3 flex flex-col min-h-0 border-4 border-red-600 shadow-[8px_8px_0px_0px_rgba(220,38,38,1)] bg-white">
              <div className="bg-red-600 text-white px-4 py-1 text-xs font-mono">
                DIAGNOSTICS
              </div>
              <div className="flex-1 bg-[#1e1e1e] text-red-400 p-4 font-mono overflow-auto whitespace-pre-wrap text-sm">
                {error}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default Playground;
