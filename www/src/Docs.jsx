import React, { useState, useEffect, useMemo } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import Fuse from 'fuse.js';
import { createHighlighter } from 'shiki';
import { 
  BrutalCard, 
  Heading, 
  Text, 
  GridLineHorizontal,
  Button,
  Input
} from 'botanical-ui';

import loftGrammar from '../../.vscode-extension/syntaxes/loft.tmLanguage.json';

const Docs = () => {
  import React, { useState, useEffect, useMemo } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import Fuse from 'fuse.js';
import { createHighlighter } from 'shiki';
import { 
  BrutalCard, 
  Heading, 
  Text, 
  GridLineHorizontal,
  Button,
  Input
} from 'botanical-ui';
import Layout from './Layout';

import loftGrammar from '../../.vscode-extension/syntaxes/loft.tmLanguage.json';

const Docs = () => {
  const { "*": path } = useParams();
  const [content, setContent] = useState('');
  const [summary, setSummary] = useState([]);
  const [loading, setLoading] = useState(true);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [docIndex, setDocIndex] = useState([]);
  const [highlighter, setHighlighter] = useState(null);
  const navigate = useNavigate();

  const docPath = path || 'introduction.md';

  useEffect(() => {
    createHighlighter({
      themes: ['tomorrow-night'],
      langs: [
        {
          ...loftGrammar,
          name: 'loft',
          aliases: ['twang']
        },
        'rust',
        'bash',
        'json'
      ]
    }).then(setHighlighter);
  }, []);

  // Fuzzy search index
  const fuse = useMemo(() => new Fuse(docIndex, {
    keys: ['title', 'content'],
    threshold: 0.3,
  }), [docIndex]);

  useEffect(() => {
    fetch('/docs/SUMMARY.md')
      .then(res => res.text())
      .then(text => {
        const lines = text.split('\n');
        const items = [];
        let currentSection = '';

        lines.forEach(line => {
          if (line.startsWith('# ')) {
            currentSection = line.replace('# ', '').trim();
          } else if (line.trim().startsWith('- [')) {
            const match = line.match(/\[(.*?)\]\((.*?)\)/);
            if (match) {
              items.push({
                title: match[1],
                path: match[2],
                section: currentSection
              });
            }
          }
        });
        setSummary(items);
      });

    // Build search index
    // In a real app, this would be pre-generated
    const buildIndex = async () => {
      // This is a simplified version - ideally we'd fetch a pre-built index
      // or fetch all docs. For now, we'll just search titles from summary
      // and maybe fetch content on demand or just search titles.
      // Let's just search titles for now to be fast.
    };
    buildIndex();
  }, []);

  useEffect(() => {
    setLoading(true);
    fetch(`/docs/${docPath}`)
      .then(res => {
        if (!res.ok) throw new Error('Doc not found');
        return res.text();
      })
      .then(text => {
        setContent(text);
        setLoading(false);
      })
      .catch(err => {
        console.error(err);
        setContent('# 404 Not Found\n\nThe requested documentation page could not be found.');
        setLoading(false);
      });
  }, [docPath]);

  useEffect(() => {
    if (summary.length > 0) {
      setDocIndex(summary.map(s => ({ ...s, content: s.title }))); // Simple index
    }
  }, [summary]);

  const searchResults = useMemo(() => {
    if (!searchQuery) return [];
    return fuse.search(searchQuery).map(r => r.item).slice(0, 5);
  }, [searchQuery, fuse]);

  return (
    <Layout fullWidth>
      <div className="flex flex-col md:flex-row min-h-[calc(100vh-64px)]">
        {/* Search Modal */}
        {searchOpen && (
          <div className="fixed inset-0 bg-bio-black/50 z-[100] flex items-start justify-center pt-20 backdrop-blur-sm" onClick={() => setSearchOpen(false)}>
            <div className="bg-bio-cream w-full max-w-xl rounded-xl shadow-2xl overflow-hidden border border-bio-black/10" onClick={e => e.stopPropagation()}>
              <div className="p-4 border-b border-bio-black/5">
                <Input 
                  autoFocus
                  placeholder="Search documentation..." 
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  className="w-full text-lg border-none focus:ring-0 bg-transparent"
                />
              </div>
              <div className="max-h-[60vh] overflow-y-auto p-2">
                {searchResults.length > 0 ? (
                  searchResults.map((result, i) => (
                    <Link 
                      key={result.path} 
                      to={`/docs/${result.path}`}
                      onClick={() => setSearchOpen(false)}
                      className={`block p-3 rounded-lg hover:bg-bio-green/10 transition-colors ${i === selectedIndex ? 'bg-bio-green/5' : ''}`}
                    >
                      <div className="font-bold text-bio-black">{result.title}</div>
                      <div className="text-xs text-bio-black/50 uppercase tracking-wider">{result.section}</div>
                    </Link>
                  ))
                ) : searchQuery ? (
                  <div className="p-4 text-center text-bio-black/50">No results found.</div>
                ) : (
                  <div className="p-4 text-center text-bio-black/50 text-sm">Type to search...</div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Sidebar */}
        <aside className="w-full md:w-64 bg-bio-offwhite border-r border-bio-black/5 p-6 md:h-[calc(100vh-64px)] md:sticky md:top-16 overflow-y-auto">
          <div className="mb-8">
            <Button 
              variant="outline" 
              className="w-full justify-between text-sm text-bio-black/60 hover:text-bio-black bg-white"
              onClick={() => setSearchOpen(true)}
            >
              <span>🔍 Search</span>
              <span className="text-xs border border-bio-black/10 px-1.5 py-0.5 rounded bg-bio-black/5">⌘K</span>
            </Button>
          </div>
          
          <div className="space-y-8">
            {Array.from(new Set(summary.map(s => s.section))).map(section => (
              <div key={section}>
                <Text variant="mono" className="text-[11px] uppercase tracking-widest font-bold text-bio-black/40 mb-3 block">
                  {section || 'General'}
                </Text>
                <ul className="space-y-1 border-l border-bio-black/5 ml-1 pl-3">
                  {summary.filter(s => s.section === section).map(item => (
                    <li key={item.path}>
                      <Link 
                        to={`/docs/${item.path}`}
                        className={`text-sm block py-1.5 transition-colors ${docPath === item.path ? 'font-bold text-bio-green -ml-3.5 pl-3.5 border-l-2 border-bio-green' : 'text-bio-black/70 hover:text-bio-black'}`}
                      >
                        {item.title}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 p-8 md:p-16 bg-white overflow-y-auto">
          <div className="max-w-3xl mx-auto">
            {loading ? (
              <div className="flex justify-center py-20">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
              </div>
            ) : (
              <article className="prose prose-bio max-w-none prose-headings:font-serif prose-headings:font-bold prose-h1:text-4xl prose-h2:text-2xl prose-a:text-bio-green prose-code:text-bio-green-dark prose-pre:bg-bio-black prose-pre:shadow-lg prose-pre:border-2 prose-pre:border-bio-black">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={{
                    h1: ({node, ...props}) => <Heading level={1} serif className="mb-8 pb-4 border-b border-gray-100" {...props} />, 
                    h2: ({node, ...props}) => <Heading level={2} className="mt-12 mb-6" {...props} />, 
                    h3: ({node, ...props}) => <Heading level={3} className="mt-8 mb-4" {...props} />, 
                    p: ({node, ...props}) => <Text variant="body" className="mb-6 leading-relaxed text-lg text-gray-700" {...props} />, 
                    code: ({node, className, children, ...props}) => {
                      const match = /language-(\w+)/.exec(className || '');
                      const lang = match ? match[1] : null;
                      const isBlock = lang || (node.position.start.line !== node.position.end.line);
                      
                      if (isBlock && highlighter) {
                        try {
                          const html = highlighter.codeToHtml(String(children).replace(/\n$/, ''), {
                            lang: lang || 'text',
                            theme: 'tomorrow-night'
                          });
                          return <div className="my-8 rounded-lg overflow-hidden shadow-lg border border-gray-800" dangerouslySetInnerHTML={{ __html: html }} />;
                        } catch (e) {
                          console.error('Shiki error:', e);
                        }
                      }

                      return isBlock ? (
                        <pre className="bg-gray-900 text-gray-100 p-6 rounded-lg font-mono text-sm overflow-x-auto my-8 shadow-lg">
                          <code className={className} {...props}>{children}</code>
                        </pre>
                      ) : (
                        <code className="bg-gray-100 px-1.5 py-0.5 rounded font-mono text-[0.9em] text-pink-600 font-medium" {...props}>{children}</code>
                      );
                    },
                    ul: ({node, ...props}) => <ul className="list-disc pl-6 mb-6 space-y-2 text-gray-700" {...props} />, 
                    li: ({node, ...props}) => <li className="text-lg" {...props} />, 
                    a: ({node, ...props}) => {
                        // Handle internal links
                        if (props.href.startsWith('./')) {
                            const target = props.href.replace('./', '');
                            return <Link to={`/docs/${target}`} className="text-bio-green hover:underline font-medium decoration-2 underline-offset-2" {...props}>{props.children}</Link>
                        }
                        return <a className="text-bio-green hover:underline font-medium decoration-2 underline-offset-2" {...props} />
                    }
                  }}
                >
                  {content}
                </ReactMarkdown>
              </article>
            )}
            
            <div className="mt-24 pt-8 border-t border-gray-100 flex justify-between items-center">
              <Button variant="ghost" size="sm" onClick={() => navigate('/')} className="text-gray-500 hover:text-bio-black">← Back to Home</Button>
              <Text variant="caption" className="text-gray-400 font-mono text-xs">LOFT LANGUAGE v0.1.0</Text>
            </div>
          </div>
        </main>
      </div>
    </Layout>;
  const [content, setContent] = useState('');
  const [summary, setSummary] = useState([]);
  const [loading, setLoading] = useState(true);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [docIndex, setDocIndex] = useState([]);
  const [highlighter, setHighlighter] = useState(null);
  const navigate = useNavigate();

  const docPath = path || 'introduction.md';

  useEffect(() => {
    createHighlighter({
      themes: ['tomorrow-night'],
      langs: [
        {
          ...loftGrammar,
          name: 'loft',
          aliases: ['twang']
        },
        'rust',
        'bash',
        'json'
      ]
    }).then(setHighlighter);
  }, []);

  // Fuzzy search index
  const fuse = useMemo(() => new Fuse(docIndex, {
    keys: ['title', 'content'],
    threshold: 0.3,
    includeMatches: true,
    ignoreLocation: true
  }), [docIndex]);

  const searchResults = useMemo(() => {
    if (!searchQuery) return [];
    const results = fuse.search(searchQuery).slice(0, 6);
    setSelectedIndex(0);
    return results;
  }, [fuse, searchQuery]);

  const getSnippet = (content, matches) => {
    if (!matches) return content.slice(0, 80) + '...';
    const contentMatch = matches.find(m => m.key === 'content');
    if (!contentMatch) return content.slice(0, 80) + '...';
    
    const [startIdx] = contentMatch.indices[0];
    const start = Math.max(0, startIdx - 30);
    const end = Math.min(content.length, startIdx + 100);
    let snippet = content.slice(start, end).replace(/\n/g, ' ');
    if (start > 0) snippet = '...' + snippet;
    if (end < content.length) snippet = snippet + '...';
    return snippet;
  };

  useEffect(() => {
    // Fetch Summary for sidebar
    fetch('/api/docs/SUMMARY.md')
      .then(res => {
        if (!res.ok) throw new Error('Not found');
        const contentType = res.headers.get('content-type');
        if (contentType && contentType.includes('text/html')) {
          throw new Error('Received HTML instead of Markdown');
        }
        return res.text();
      })
      .then(text => {
        const lines = text.split('\n');
        const items = [];
        let currentSection = '';
        
        lines.forEach(line => {
          if (line.startsWith('# ')) {
            currentSection = line.replace('# ', '');
          } else if (line.includes('[') && line.includes('](')) {
            const match = line.match(/\[(.*?)\]\(\.\/(.*?)\)/);
            if (match) {
              items.push({
                title: match[1],
                path: match[2],
                section: currentSection
              });
            }
          }
        });
        setSummary(items);
        
        // Background: Index all pages for search
        items.forEach(item => {
          fetch(`/api/docs/${item.path}`)
            .then(res => res.text())
            .then(content => {
              setDocIndex(prev => [...prev, {
                title: item.title,
                path: item.path,
                content: content.replace(/[#*`]/g, '') // Strip markdown for cleaner search
              }]);
            });
        });
      });
  }, []);

  useEffect(() => {
    const handleKeyDown = (e) => {
      if (e.key.toLowerCase() === 's' && !searchOpen && document.activeElement.tagName !== 'INPUT') {
        e.preventDefault();
        setSearchOpen(true);
      }
      if (e.key === 'Escape') {
        setSearchOpen(false);
      }
      
      if (searchOpen) {
        if (e.key === 'ArrowDown') {
          e.preventDefault();
          setSelectedIndex(prev => (prev + 1) % Math.max(1, searchResults.length));
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          setSelectedIndex(prev => (prev - 1 + searchResults.length) % Math.max(1, searchResults.length));
        } else if (e.key === 'Enter' && searchResults[selectedIndex]) {
          e.preventDefault();
          const item = searchResults[selectedIndex].item;
          navigate(`/docs/${item.path}`);
          setSearchOpen(false);
          setSearchQuery('');
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [searchOpen, searchResults, selectedIndex, navigate]);

  useEffect(() => {
    setLoading(true);
    fetch(`/api/docs/${docPath}`)
      .then(res => {
        if (!res.ok) throw new Error('Not found');
        const contentType = res.headers.get('content-type');
        if (contentType && contentType.includes('text/html')) {
          throw new Error('Received HTML instead of Markdown. Is the backend running?');
        }
        return res.text();
      })
      .then(text => {
        setContent(text);
        setLoading(false);
      })
      .catch(() => {
        setContent('# 404\nDocument not found.');
        setLoading(false);
      });
  }, [docPath]);

  return (
    <div className="min-h-screen bg-bio-cream font-sans text-bio-black relative">
      <style dangerouslySetInnerHTML={{ __html: `
        .shiki {
          padding: 1.5rem;
          margin: 0;
          font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
          font-size: 0.875rem;
          line-height: 1.5;
          overflow-x: auto;
        }
      `}} />
      {/* Search Modal */}
      {searchOpen && (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-32 px-4 bg-bio-black/20 backdrop-blur-[2px]" onClick={() => setSearchOpen(false)}>
          <BrutalCard 
            className="w-full max-w-xl bg-bio-cream p-0 overflow-hidden shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] border-2"
            onClick={e => e.stopPropagation()}
          >
            <div className="p-3 border-b-2 border-bio-black flex items-center gap-2 bg-bio-offwhite">
              <span className="text-lg opacity-50">🔍</span>
              <input 
                autoFocus
                placeholder="Search docs..."
                className="flex-1 bg-transparent border-none outline-none text-base font-mono placeholder:opacity-30"
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
              />
              <div className="text-[10px] border border-bio-black/20 px-1 rounded opacity-40">ESC</div>
            </div>
            <div className="max-h-[50vh] overflow-y-auto">
              {searchResults.length > 0 ? (
                searchResults.map(({ item, matches }, index) => (
                  <div 
                    key={item.path}
                    className={`p-3 cursor-pointer border-b border-bio-black/5 last:border-none transition-all group ${
                      index === selectedIndex ? 'bg-bio-green/10 border-l-4 border-l-bio-green' : 'hover:bg-bio-green/5'
                    }`}
                    onClick={() => {
                      navigate(`/docs/${item.path}`);
                      setSearchOpen(false);
                      setSearchQuery('');
                    }}
                  >
                    <div className="flex justify-between items-center mb-1">
                      <Heading level={4} className={`text-sm ${index === selectedIndex ? 'text-bio-green' : 'group-hover:text-bio-green'}`}>{item.title}</Heading>
                      <Text variant="mono" className="text-[10px] opacity-30">{item.path}</Text>
                    </div>
                    <Text className="text-xs opacity-60 line-clamp-2 font-serif italic">
                      {getSnippet(item.content, matches)}
                    </Text>
                  </div>
                ))
              ) : searchQuery ? (
                <div className="p-8 text-center opacity-40 text-sm italic">No results found for "{searchQuery}"</div>
              ) : (
                <div className="p-8 text-center opacity-40 text-sm">Type to search...</div>
              )}
            </div>
          </BrutalCard>
        </div>
      )}

      <div className="flex flex-col md:flex-row min-h-screen">
        {/* Sidebar */}
        <aside className="w-full md:w-72 border-r border-bio-black p-6 bg-bio-offwhite md:h-screen md:sticky md:top-0 overflow-y-auto">
          <Link to="/" className="block mb-8">
            <Heading level={2} serif className="hover:text-bio-green transition-colors">loft</Heading>
          </Link>

          <div className="mb-6">
            <Button 
              variant="outline" 
              size="sm" 
              className="w-full justify-start gap-2 font-mono text-[10px] uppercase tracking-tighter opacity-70 hover:opacity-100"
              onClick={() => setSearchOpen(true)}
            >
              <span>🔍</span> Search <span className="ml-auto border border-bio-black/20 px-1 rounded">S</span>
            </Button>
          </div>
          
          <div className="space-y-6">
            {Array.from(new Set(summary.map(s => s.section))).map(section => (
              <div key={section}>
                <Text variant="mono" className="text-[10px] uppercase tracking-widest opacity-40 mb-2 block">
                  {section || 'General'}
                </Text>
                <ul className="space-y-1">
                  {summary.filter(s => s.section === section).map(item => (
                    <li key={item.path}>
                      <Link 
                        to={`/docs/${item.path}`}
                        className={`text-sm hover:underline block py-1 transition-colors ${docPath === item.path ? 'font-bold text-bio-green' : 'opacity-70 hover:opacity-100'}`}
                      >
                        {item.title}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 p-8 md:p-16 bg-bio-cream overflow-y-auto">
          <div className="max-w-3xl">
            {loading ? (
              <Text>Loading documentation...</Text>
            ) : (
              <article className="prose prose-bio max-w-none">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={{
                    h1: ({node, ...props}) => <Heading level={1} serif className="mb-10 border-b-2 border-bio-black/10 pb-6" {...props} />,
                    h2: ({node, ...props}) => <Heading level={2} className="mt-16 mb-6 border-l-4 border-bio-green pl-4" {...props} />,
                    h3: ({node, ...props}) => <Heading level={3} className="mt-10 mb-4" {...props} />,
                    p: ({node, ...props}) => <Text variant="body" className="mb-6 leading-relaxed text-lg" {...props} />,
                    code: ({node, className, children, ...props}) => {
                      const match = /language-(\w+)/.exec(className || '');
                      const lang = match ? match[1] : null;
                      const isBlock = lang || (node.position.start.line !== node.position.end.line);
                      
                      if (isBlock && highlighter) {
                        try {
                          const html = highlighter.codeToHtml(String(children).replace(/\n$/, ''), {
                            lang: lang || 'text',
                            theme: 'tomorrow-night'
                          });
                          return <div className="my-8 border-2 border-bio-black shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] rounded-lg overflow-hidden" dangerouslySetInnerHTML={{ __html: html }} />;
                        } catch (e) {
                          console.error('Shiki error:', e);
                        }
                      }

                      return isBlock ? (
                        <pre className="bg-bio-black text-bio-cream p-6 rounded-lg font-mono text-sm overflow-x-auto my-8 border-2 border-bio-black shadow-[6px_6px_0px_0px_rgba(0,0,0,1)]">
                          <code className={className} {...props}>{children}</code>
                        </pre>
                      ) : (
                        <code className="bg-bio-green/10 px-1.5 py-0.5 rounded font-mono text-[0.9em] border border-bio-green/20" {...props}>{children}</code>
                      );
                    },
                    ul: ({node, ...props}) => <ul className="list-disc pl-6 mb-6 space-y-3" {...props} />,
                    li: ({node, ...props}) => <li className="text-bio-black text-lg" {...props} />,
                    a: ({node, ...props}) => {
                        // Handle internal links
                        if (props.href.startsWith('./')) {
                            const target = props.href.replace('./', '');
                            return <Link to={`/docs/${target}`} className="text-bio-green hover:underline font-bold decoration-2 underline-offset-4" {...props}>{props.children}</Link>
                        }
                        return <a className="text-bio-green hover:underline font-bold decoration-2 underline-offset-4" {...props} />
                    }
                  }}
                >
                  {content}
                </ReactMarkdown>
              </article>
            )}
            
            <div className="mt-24 pt-12 border-t border-bio-black/10 flex justify-between items-center">
              <Button variant="ghost" size="sm" onClick={() => navigate('/')}>← Back to Home</Button>
              <Text variant="caption" className="opacity-30 font-mono text-[10px]">LOFT LANGUAGE v0.1.0</Text>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
};

export default Docs;
