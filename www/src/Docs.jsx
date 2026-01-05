import React, { useState, useEffect, useMemo } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { createHighlighter } from 'shiki';
import Fuse from 'fuse.js';
import { Button, Heading, Text, Input, BrutalCard } from 'botanical-ui';
import Layout from './Layout';
import loftGrammar from './loft.tmLanguage.json';

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
          aliases: ['loft']
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
              className="w-full justify-between text-sm !text-bio-black !bg-white hover:!bg-bio-offwhite border border-bio-black/10 shadow-sm"
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
                        to={`/book/${item.path}`}
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
                                    to={`/docs/${fullPath}`} 
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
              <Button variant="ghost" size="sm" onClick={() => navigate('/')} className="text-gray-500 hover:text-bio-black">← Back to Home</Button>
              <Text variant="caption" className="text-gray-400 font-mono text-xs">LOFT LANGUAGE v0.1.0</Text>
            </div>
          </div>
        </main>
      </div>
    </Layout>
  );
};

export default Docs;
