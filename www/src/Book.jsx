import React, { useState, useEffect, useMemo } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { createHighlighter } from 'shiki';
import Fuse from 'fuse.js';
import { Heading, Text, Input, BrutalCard } from 'botanical-ui';
import BrutalButton from './BrutalButton';
import Layout from './Layout';
import loftGrammar from './loft.tmLanguage.json';

const Book = () => {
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

  const rawPath = path || 'introduction';
  const docPath = rawPath.endsWith('.md') ? rawPath : rawPath + '.md';

  useEffect(() => {
    createHighlighter({
      themes: ['one-dark-pro'],
      langs: [
        {
          ...loftGrammar,
          name: 'loft',
          aliases: ['lf']
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
    minMatchCharLength: 2,
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
                path: match[2].startsWith('./') ? match[2].substring(2) : match[2],
                section: currentSection
              });
            }
          }
        });
        setSummary(items);
      });
  }, []);

  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setSearchOpen(true);
      }
      if (e.key === 'Escape') {
        setSearchOpen(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
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
      // Initially set simple index
      setDocIndex(summary.map(s => ({ ...s, content: s.title })));
      
      // Load full content for fuzzy search
      const loadFullContent = async () => {
        const fullIndex = await Promise.all(
          summary.map(async (item) => {
            try {
              const res = await fetch(`/docs/${item.path}`);
              const text = await res.text();
              // Strip markdown if possible for better search results, but keeping it simple for now
              return { ...item, content: text };
            } catch (e) {
              return { ...item, content: item.title };
            }
          })
        );
        setDocIndex(fullIndex);
      };
      
      loadFullContent();
    }
  }, [summary]);

  const searchResults = useMemo(() => {
    if (!searchQuery) return [];
    return fuse.search(searchQuery).slice(0, 8);
  }, [searchQuery, fuse]);

  const highlightMatch = (text, indices) => {
    if (!indices || indices.length === 0) return text;
    const parts = [];
    let lastIndex = 0;
    const sorted = [...indices].sort((a, b) => a[0] - b[0]);
    sorted.forEach(([start, end], i) => {
      parts.push(text.substring(lastIndex, start));
      parts.push(<mark key={i} className="bg-bio-green/30 text-bio-green-dark font-bold rounded px-0.5">{text.substring(start, end + 1)}</mark>);
      lastIndex = end + 1;
    });
    parts.push(text.substring(lastIndex));
    return parts;
  };

  const getSnippet = (content, matches) => {
    const contentMatch = matches.find(m => m.key === 'content');
    if (!contentMatch) return { text: content.substring(0, 100) + (content.length > 100 ? '...' : ''), indices: [] };
    
    // Find the first match in content (not title)
    const firstMatch = contentMatch.indices[0];
    const start = Math.max(0, firstMatch[0] - 50);
    const end = Math.min(content.length, firstMatch[1] + 100);
    
    let snippet = content.substring(start, end).replace(/\n/g, ' ');
    const offset = start > 0 ? 3 : 0;
    
    const adjustedIndices = contentMatch.indices
      .filter(([s, e]) => s >= start && e <= end)
      .map(([s, e]) => [s - start + offset, e - start + offset]);
      
    return {
      text: (start > 0 ? '...' : '') + snippet + (end < content.length ? '...' : ''),
      indices: adjustedIndices
    };
  };

  return (
    <Layout fullWidth>
      <div className="flex flex-col md:flex-row min-h-[calc(100vh-64px)]">
        {/* Search Modal */}
        {searchOpen && (
          <div className="fixed inset-0 bg-bio-black/50 z-100 flex items-start justify-center pt-20 backdrop-blur-sm" onClick={() => setSearchOpen(false)}>
            <div className="bg-bio-cream w-full max-w-xl rounded-xl shadow-2xl overflow-hidden border border-bio-black/10" onClick={e => e.stopPropagation()}>
              <div className="p-4 border-b border-bio-black/5">
                <input 
                  autoFocus
                  placeholder="Search documentation..." 
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  className="w-full text-lg border-none focus:ring-0 bg-transparent outline-none"
                />
              </div>
              <div className="max-h-[60vh] overflow-y-auto p-2">
                {searchResults.length > 0 ? (
                  searchResults.map((result, i) => {
                    const item = result.item;
                    const titleMatch = result.matches.find(m => m.key === 'title');
                    const snippet = getSnippet(item.content, result.matches);

                    return (
                      <Link 
                        key={item.path} 
                        to={`/book/${item.path.replace(/\.md$/, '')}`}
                        onClick={() => setSearchOpen(false)}
                        className={`block p-4 rounded-lg hover:bg-bio-green/10 transition-colors border-2 mb-2 ${i === selectedIndex ? 'bg-bio-green/5 border-bio-green' : 'border-transparent'}`}
                      >
                        <div className="flex justify-between items-start mb-1">
                          <div className="font-bold text-bio-black text-lg">
                            {titleMatch ? highlightMatch(item.title, titleMatch.indices) : item.title}
                          </div>
                          <div className="text-[10px] bg-bio-black/5 text-bio-black/40 px-2 py-0.5 rounded font-mono uppercase tracking-tighter">
                            {item.section || 'General'}
                          </div>
                        </div>
                        <div className="text-sm text-gray-600 line-clamp-2 font-mono bg-white/50 p-2 rounded border border-bio-black/5">
                          {highlightMatch(snippet.text, snippet.indices)}
                        </div>
                      </Link>
                    );
                  })
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
        <aside className="min-h-screen w-full md:w-64 bg-bio-offwhite border-r border-bio-black/5 p-6 md:sticky md:top-16 self-start">
          <div className="mb-8">
            <BrutalButton 
              variant="outline" 
              className="w-full justify-between text-sm shadow-sm"
              onClick={() => setSearchOpen(true)}
            >
              <span>🔍 Search</span>
              <span className="text-xs border border-bio-black/10 px-1.5 py-0.5 rounded bg-bio-black/5">⌘K</span>
            </BrutalButton>
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
                        to={`/book/${item.path.replace(/\.md$/, '')}`}
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
                    pre: ({node, children}) => <>{children}</>,
                    code: ({node, className, children, ...props}) => {
                      const match = /language-(\w+)/.exec(className || '');
                      const lang = match ? match[1] : null;
                      const isBlock = !!lang || String(children).includes('\n');
                      
                      if (isBlock && highlighter) {
                        try {
                          const html = highlighter.codeToHtml(String(children).replace(/\n$/, ''), {
                            lang: lang || 'text',
                            theme: 'one-dark-pro'
                          });
                          return <div className="my-8 rounded-lg overflow-hidden shadow-lg border border-gray-800" dangerouslySetInnerHTML={{ __html: html }} />;
                        } catch (e) {
                          console.error('Shiki error:', e);
                        }
                      }

                      if (!isBlock && highlighter) {
                        try {
                          const html = highlighter.codeToHtml(String(children), {
                            lang: 'loft',
                            theme: 'one-dark-pro'
                          });
                          const innerMatch = html.match(/<code[^>]*>([\s\S]*)<\/code>/);
                          const innerHtml = innerMatch ? innerMatch[1] : null;
                          if (innerHtml) {
                            return <code
                              className="inline rounded font-mono text-[0.9em] font-medium"
                              style={{ background: '#282c34', padding: '0.2em 0.4em' }}
                              dangerouslySetInnerHTML={{ __html: innerHtml }}
                            />;
                          }
                        } catch (e) {
                          // fall through to plain inline code
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
                        const isExternal = props.href.startsWith('http') || props.href.startsWith('//');
                        if (!isExternal) {
                            let target = props.href;
                            if (target.startsWith('./')) {
                                target = target.replace('./', '');
                            }
                            
                            // Resolve relative path
                            const currentDir = docPath.includes('/') 
                                ? docPath.substring(0, docPath.lastIndexOf('/') + 1) 
                                : '';
                            
                            const fullPath = target.startsWith('/') 
                                ? target.substring(1) 
                                : currentDir + target;

                            return (
                                <Link 
                                    to={`/book/${fullPath.replace(/\.md$/, '')}`} 
                                    className="text-bio-green hover:underline font-medium decoration-2 underline-offset-2"
                                >
                                    {props.children}
                                </Link>
                            );
                        }
                        return (
                            <a 
                                href={props.href}
                                target="_blank" 
                                rel="noreferrer" 
                                className="text-bio-green hover:underline font-medium decoration-2 underline-offset-2"
                            >
                                {props.children}
                            </a>
                        );
                    }
                  }}
                >
                  {content}
                </ReactMarkdown>
              </article>
            )}
            
            <div className="mt-24 pt-8 border-t border-gray-100 flex justify-between items-center">
              <BrutalButton variant="ghost" size="sm" onClick={() => navigate('/')}>← Back to Home</BrutalButton>
              <div className="flex items-center gap-4">
                <a
                  href={`https://github.com/fargonesh/loft/edit/main/book/src/${docPath}`}
                  target="_blank"
                  rel="noreferrer"
                  className="text-xs text-gray-400 hover:text-bio-green transition-colors font-mono flex items-center gap-1"
                >
                  ✏️ Edit on GitHub
                </a>
                <Text variant="caption" className="text-gray-400 font-mono text-xs">LOFT LANGUAGE v0.1.0</Text>
              </div>
            </div>
          </div>
        </main>
      </div>
    </Layout>
  );
};

export default Book;
