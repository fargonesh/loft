import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import Layout from './Layout';
import BrutalButton from './BrutalButton';
import { Heading, Text } from 'botanical-ui';
import Editor from '@monaco-editor/react';
import { loftLanguage, loftTheme } from './monacoConfig';
import examples from 'virtual:loft-examples';

const Playground = () => {
  const navigate = useNavigate();
  const [code, setCode] = useState(`term.println("Hello, World!");
term.println("Check this out: " + (10 * 20));

fn factorial(n: num) -> num {
  if (n <= 1) { return 1; }
  return n * factorial(n - 1);
}

term.println("Factorial of 5: " + factorial(5));
`);
  const [output, setOutput] = useState('');
  const [isWasmLoaded, setIsWasmLoaded] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const workerRef = useRef(null);
  const pendingRef = useRef(new Map());
  const counterRef = useRef(0);
  // spawnWorkerRef is a ref so onerror can recursively restart without stale closures
  const spawnWorkerRef = useRef(null);

  spawnWorkerRef.current = useCallback(() => {
    const worker = new Worker(
      new URL('./wasmWorker.js', import.meta.url),
      { type: 'module' }
    );

    worker.onmessage = ({ data }) => {
      const { id, result, error } = data;

      if (id === '__ready') {
        if (error) {
          setOutput(`Failed to load runtime: ${error}`);
          setIsLoading(false);
        } else {
          setIsWasmLoaded(true);
          setIsLoading(false);
          // Replace the "Restarting…" message written by handleRun on crash
          setOutput(prev =>
            prev.includes('> Restarting runtime')
              ? prev.replace('> Restarting runtime…', '> Runtime restarted. Ready.')
              : prev
          );
        }
        return;
      }

      const pending = pendingRef.current.get(id);
      if (pending) {
        pendingRef.current.delete(id);
        if (error !== undefined) {
          pending.reject(new Error(error));
        } else {
          pending.resolve(result);
        }
      }
    };

    worker.onerror = () => {
      // Worker crashed due to a WASM panic/abort.
      // Reject any in-flight call so the UI unblocks.
      for (const [, pending] of pendingRef.current) {
        pending.reject(new Error('Runtime crashed (panic). Restarting…'));
      }
      pendingRef.current.clear();

      setIsWasmLoaded(false);
      setIsLoading(true);
      worker.terminate();
      workerRef.current = spawnWorkerRef.current();
    };

    return worker;
  }, []);



  const handleEditorWillMount = (monaco) => {
    monaco.languages.register({ id: 'loft' });
    monaco.languages.setMonarchTokensProvider('loft', loftLanguage);
    monaco.editor.defineTheme('loftTheme', loftTheme);
  }

  useEffect(() => {
    workerRef.current = spawnWorkerRef.current();
    return () => {
      workerRef.current?.terminate();
    };
  }, []);

  const callWorker = useCallback((type, code) => {
    return new Promise((resolve, reject) => {
      const id = ++counterRef.current;
      pendingRef.current.set(id, { resolve, reject });
      workerRef.current.postMessage({ id, type, code });
    });
  }, []);

  const handleRun = async () => {
    if (!isWasmLoaded) return;
    try {
      const result = await callWorker('run', code);
      setOutput(result);
    } catch (e) {
      // Worker crashed (WASM panic/abort) — onerror already respawns it.
      // The new worker's __ready handler will swap this message when ready.
      setOutput(`Runtime Error: ${e.message}\n\n> Restarting runtime…`);
    }
  };

  const handleFormat = async () => {
    if (!isWasmLoaded) return;
    try {
      const result = await callWorker('format', code);
      setCode(result);
    } catch (e) {
      setOutput(`Format error: ${e.message}`);
    }
  };

  return (
    <Layout fullWidth>
      <div className="flex flex-col md:flex-row min-h-[calc(100vh-64px)] bg-bio-cream overflow-hidden">
        
        {/* Sidebar */}
        <aside className="min-h-screen w-full md:w-64 bg-bio-offwhite border-r border-bio-black/10 p-6 md:h-[calc(100vh-64px)] md:sticky md:top-16 overflow-y-auto shrink-0 shadow-inner">
          <Heading level={4} serif className="text-sm uppercase tracking-widest text-bio-black opacity-50 mb-6">Examples</Heading>
          
          <div className="space-y-3">
            {examples.map((example) => (
              <div key={example.name} className="relative group">
                <button
                  onClick={() => setCode(example.code)}
                  className="w-full text-left p-4 rounded-lg border-2 border-transparent hover:border-bio-green/20 hover:bg-white transition-all group shadow-sm bg-white/50"
                >
                  <div className="font-bold text-bio-black group-hover:text-bio-green transition-colors pr-6">{example.name}</div>
                  <div className="text-[11px] text-bio-black/60 mt-1 leading-tight">{example.description}</div>
                </button>
                <a
                  href={`https://github.com/fargonesh/loft/blob/main/examples/${example.file}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={e => e.stopPropagation()}
                  title="View on GitHub"
                  className="absolute top-3 right-3 opacity-0 group-hover:opacity-60 hover:!opacity-100 transition-opacity text-bio-black"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
                  </svg>
                </a>
              </div>
            ))}
          </div>

          <div className="mt-12 pt-8 border-t border-bio-black/5">
             <Heading level={4} serif className="text-sm uppercase tracking-widest text-bio-black opacity-50 mb-4">Resources</Heading>
             <ul className="space-y-3">
               <li>
                 <button onClick={() => navigate('/book/introduction.md')} className="text-sm font-medium text-bio-black/70 hover:text-bio-green transition-colors">
                   Book →
                 </button>
               </li>
               <li>
                 <button onClick={() => navigate('/d/std')} className="text-sm font-medium text-bio-black/70 hover:text-bio-green transition-colors">
                   Standard Library →
                 </button>
               </li>
             </ul>
          </div>
        </aside>

        {/* Main Workspace */}
        <main className="flex-1 flex flex-col p-4 md:p-10 overflow-auto">
          <div className="max-w-5xl mx-auto w-full flex flex-col h-full">
            <div className="mb-8">
              <Heading level={1} serif className="mb-2">Playground</Heading>
              <Text className="text-lg text-bio-black/60">
                The loft playground is a secure, sandboxed environment where you can write, format, and run loft code directly in your browser.
              </Text>
            </div>

            {/* Editor & Terminal Container */}
            <div className="flex-1 flex flex-col bg-white rounded-xl border-2 border-bio-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] overflow-hidden min-h-[600px]">
              
              {/* Toolbar */}
              <div className="bg-bio-offwhite border-b-2 border-bio-black px-6 py-3 flex justify-between items-center shrink-0">
                <div className="flex items-center gap-2">
                  <div className="flex gap-1.5 mr-4">
                    <div className="w-3 h-3 rounded-full bg-red-400 border border-bio-black/20"></div>
                    <div className="w-3 h-3 rounded-full bg-yellow-400 border border-bio-black/20"></div>
                    <div className="w-3 h-3 rounded-full bg-green-400 border border-bio-black/20"></div>
                  </div>
                  <Text variant="mono" className="text-xs font-bold uppercase tracking-wider text-bio-black/40">main.lf</Text>
                </div>
                
                <div className="flex items-center gap-3">
                  <BrutalButton 
                    variant="outline" 
                    size="sm" 
                    onClick={handleFormat}
                    disabled={isLoading}
                    className="h-9 px-4"
                  >
                    Format
                  </BrutalButton>
                  <BrutalButton 
                    variant="primary" 
                    size="sm" 
                    onClick={handleRun}
                    disabled={isLoading}
                    className="h-9 px-6 bg-bio-green hover:bg-bio-green-light"
                  >
                    {isLoading ? 'Loading...' : 'Run ▶'}
                  </BrutalButton>
                </div>
              </div>

              {/* Editor Block */}
              <div className="flex-1 relative min-h-[300px]">
                <Editor
                  height="100%"
                  defaultLanguage="loft"
                  theme="loftTheme"
                  value={code}
                  onChange={(value) => setCode(value || '')}
                  beforeMount={handleEditorWillMount}
                  options={{
                    minimap: { enabled: false },
                    scrollBeyondLastLine: false,
                    fontSize: 14,
                    fontFamily: "'JetBrains Mono', 'Fira Code', 'Roboto Mono', monospace",
                    padding: { top: 20, bottom: 20 },
                    automaticLayout: true,
                    lineNumbers: 'on',
                    renderWhitespace: 'none',
                    selectionHighlight: true,
                    occurrencesHighlight: true,
                    hideCursorInOverviewRuler: true,
                    overviewRulerLanes: 0,
                  }}
                />
              </div>

              {/* Terminal Section */}
              <div className="h-56 border-t-2 border-bio-black flex flex-col bg-bio-black overflow-hidden shrink-0">
                <div className="bg-white/5 px-4 py-1.5 border-b border-white/5 flex justify-between items-center shrink-0">
                  <Text variant="mono" className="text-[10px] uppercase font-bold text-white/40 tracking-widest">Compiler Output</Text>
                  {output && (
                    <button onClick={() => setOutput('')} className="text-[10px] text-white/40 hover:text-white uppercase transition-colors">Clear</button>
                  )}
                </div>
                <div className="flex-1 overflow-auto p-6 font-mono text-sm text-green-400 leading-relaxed selection:bg-white/20">
                  <pre className="whitespace-pre-wrap">
                    {output || (isLoading ? '> Initializing compiler runtime...' : '> Ready to execute loft v0.1.0-rc3')}
                  </pre>
                </div>
              </div>
            </div>

            <div className="mt-12 flex items-center justify-between opacity-40 grayscale group hover:grayscale-0 hover:opacity-100 transition-all">
              <div className="flex items-center gap-6">
                 <Text variant="caption">Powered by WASM</Text>
                 <Text variant="caption">Isolated Runtime</Text>
              </div>
              <BrutalButton variant="ghost" size="sm" onClick={() => navigate('/')}>← Back</BrutalButton>
            </div>
          </div>
        </main>

      </div>
    </Layout>
  );
};

export default Playground;
