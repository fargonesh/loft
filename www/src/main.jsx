import { useState, useEffect, useMemo } from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';
import Fuse from 'fuse.js';
import {
  BrutalCard,
  Input,
  Heading,
  Text,
  GridLineHorizontal,
} from 'botanical-ui';
import BrutalButton from './BrutalButton';
import { Link } from 'react-router-dom';

import 'botanical-ui/style.css';
import './index.css';

import AuthCallback from './AuthCallback';
import Dashboard from './Dashboard';
import Book from './Book';
import PackageInfo from './PackageInfo';
import Docs from './Docs';
import Playground from './Playground';
import Layout from './Layout';
import NotFound from './NotFound';

const Home = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [packages, setPackages] = useState([]);
  const [allPackages, setAllPackages] = useState([]);
  const [docIndex, setDocIndex] = useState([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchAllPackages();
    fetchDocsIndex();
  }, []);

  const fetchDocsIndex = async () => {
    try {
      const summaryRes = await fetch('/docs/SUMMARY.md');
      const summaryText = await summaryRes.text();
      const items = [];
      const lines = summaryText.split('\n');
      lines.forEach(line => {
        // Match both "- [title](path)" and "[title](path)"
        const match = line.match(/\[(.*?)\]\((.*?)\)/);
        if (match) {
          items.push({
            title: match[1],
            path: match[2].startsWith('./') ? match[2].substring(2) : match[2]
          });
        }
      });

      const fullIndex = await Promise.all(
        items.map(async (item) => {
          try {
            const res = await fetch(`/docs/${item.path}`);
            const text = await res.text();
            return { ...item, content: text };
          } catch (e) {
            return { ...item, content: '' };
          }
        })
      );
      setDocIndex(fullIndex);
    } catch (e) {
      console.error('Failed to index docs:', e);
    }
  };

  const fetchAllPackages = async () => {
    setLoading(true);
    try {
      const response = await fetch('/packages');
      if (response.ok) {
        const data = await response.json();
        setAllPackages(data);
      }
    } catch (error) {
      console.error('Failed to fetch packages:', error);
    } finally {
      setLoading(false);
    }
  };

  const docFuse = useMemo(() => new Fuse(docIndex, {
    keys: ['title', 'content'],
    threshold: 0.3,
    includeMatches: true
  }), [docIndex]);

  const docResults = useMemo(() => {
    if (!searchQuery || docIndex.length === 0) return [];
    return docFuse.search(searchQuery).slice(0, 5);
  }, [searchQuery, docFuse, docIndex]);

  const getDocSnippet = (content, matches) => {
    const contentMatch = matches.find(m => m.key === 'content');
    if (!contentMatch) return { text: content.substring(0, 80) + (content.length > 80 ? '...' : ''), indices: [] };
    const firstMatch = contentMatch.indices[0];
    const start = Math.max(0, firstMatch[0] - 40);
    const end = Math.min(content.length, firstMatch[1] + 60);
    const snippet = content.substring(start, end).replace(/\n/g, ' ');
    const offset = start > 0 ? 3 : 0;
    return {
      text: (start > 0 ? '...' : '') + snippet + (end < content.length ? '...' : ''),
      indices: contentMatch.indices
        .filter(([s, e]) => s >= start && e <= end)
        .map(([s, e]) => [s - start + offset, e - start + offset])
    };
  };

  const fuse = useMemo(() => new Fuse(allPackages, {
    keys: ['name', 'description'],
    threshold: 0.3,
    includeMatches: true
  }), [allPackages]);

  const searchResults = useMemo(() => {
    if (!searchQuery) return allPackages;
    return fuse.search(searchQuery).map(r => ({
      ...r.item,
      matches: r.matches
    }));
  }, [searchQuery, fuse, allPackages]);

  const highlightMatch = (text, matches, key) => {
    if (!matches) return text;
    // Handle both cases where matches is the result of fuse.search result or our mapped result
    const match = Array.isArray(matches) ? matches.find(m => m.key === key) : null;
    if (!match) return text;

    const parts = [];
    let lastIndex = 0;
    const sorted = [...match.indices].sort((a, b) => a[0] - b[0]);
    
    sorted.forEach(([start, end], i) => {
      parts.push(text.substring(lastIndex, start));
      parts.push(<mark key={i} className="bg-bio-green/30 text-bio-black rounded px-0.5">{text.substring(start, end + 1)}</mark>);
      lastIndex = end + 1;
    });
    parts.push(text.substring(lastIndex));
    return parts;
  };

  return (
    <Layout>
      <div className="text-center mb-16 pt-8">
        <Heading level={1} serif className="mb-6 text-5xl md:text-6xl text-bio-green-dark">
          Build reliability<br />with <span className="text-bio-green">loft</span>
        </Heading>
        <Text variant="body" className="mb-10 text-xl max-w-2xl mx-auto text-gray-600">
          A fresh, friendly, and expressive language that makes coding systems feel like a breeze.
        </Text>

        <div className="flex max-w-xl mx-auto gap-3 p-2 bg-white rounded-xl shadow-lg border border-gray-100">
          <Input
            placeholder="Find a package..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 border-none focus:ring-0 text-lg"
          />
          <BrutalButton onClick={() => {}}>Search</BrutalButton>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        <div className="md:col-span-2 space-y-6">
          <div className="flex items-center justify-between mb-4">
            <Heading level={2} className="text-2xl">
              {searchQuery ? 'Search Results' : 'Popular Packages'}
            </Heading>
            <Link to="/packages" className="text-bio-green hover:underline font-medium">View all</Link>
          </div>

          {loading ? (
            <div className="flex justify-center py-12">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
            </div>
          ) : (searchResults.length > 0 || docResults.length > 0) ? (
            <div className="grid gap-6">
              {/* Package Results */}
              {searchResults.length > 0 && searchResults.length < allPackages.length && (
                <div className="space-y-4">
                  <Heading level={4} className="text-xs uppercase tracking-widest text-gray-400">Packages</Heading>
                  {searchResults.map((pkg) => (
                    <div key={pkg.name} className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm hover:shadow-md transition-all duration-200 group">
                      <div className="flex justify-between items-start mb-3">
                        <Heading level={3} className="text-xl">
                          <Link to={`/p/${pkg.name}`} className="group-hover:text-bio-green transition-colors">
                            {highlightMatch(pkg.name, pkg.matches, 'name')}
                          </Link>
                        </Heading>
                        <span className="text-xs font-mono bg-bio-green/10 text-bio-green px-2 py-1 rounded-full">
                          v{pkg.version}
                        </span>
                      </div>
                      <Text variant="body" className="mb-4 text-gray-600 line-clamp-2">
                        {highlightMatch(pkg.description || 'No description provided.', pkg.matches, 'description')}
                      </Text>

                      <div className="flex items-center justify-between mt-4 pt-4 border-t border-gray-50">
                        <div className="flex gap-4 text-sm text-gray-500">
                          {pkg.authors && pkg.authors.length > 0 && (
                            <span>{pkg.authors[0]}</span>
                          )}
                          {pkg.license && <span>{pkg.license}</span>}
                        </div>
                        <Link to={`/d/${pkg.name}`} className="text-sm font-medium text-bio-green hover:underline">
                          View Docs →
                        </Link>
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {/* Documentation Results in Main Column */}
              {searchQuery && docResults.length > 0 && (
                <div className="space-y-4">
                  <Heading level={4} className="text-xs uppercase tracking-widest text-gray-400">Documentation</Heading>
                  {docResults.map((result) => {
                    const snippet = getDocSnippet(result.item.content, result.matches);
                    return (
                      <div key={result.item.path} className="bg-bio-cream/30 p-6 rounded-xl border border-bio-green/10 shadow-sm hover:shadow-md transition-all duration-200 group">
                        <Heading level={3} className="text-xl mb-2">
                          <Link to={`/book/${result.item.path.replace(/\.md$/, '')}`} className="group-hover:text-bio-green transition-colors">
                            {result.item.title}
                          </Link>
                        </Heading>
                        <div className="text-sm text-gray-600 font-mono bg-white/80 p-3 rounded border border-bio-black/5 leading-relaxed">
                          {highlightMatch(snippet.text, [{ key: 'text', indices: snippet.indices }], 'text')}
                        </div>
                        <div className="mt-4 flex justify-end">
                          <Link to={`/book/${result.item.path.replace(/\.md$/, '')}`} className="text-sm font-bold text-bio-green hover:underline">
                            Read more →
                          </Link>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}

              {/* Default view when no search */}
              {!searchQuery && allPackages.map((pkg) => (
                <div key={pkg.name} className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm hover:shadow-md transition-all duration-200 group">
                  <div className="flex justify-between items-start mb-3">
                    <Heading level={3} className="text-xl">
                      <Link to={`/p/${pkg.name}`} className="group-hover:text-bio-green transition-colors">
                        {pkg.name}
                      </Link>
                    </Heading>
                    <span className="text-xs font-mono bg-bio-green/10 text-bio-green px-2 py-1 rounded-full">
                      v{pkg.version}
                    </span>
                  </div>
                  <Text variant="body" className="mb-4 text-gray-600 line-clamp-2">{pkg.description || 'No description provided.'}</Text>
                  
                  <div className="flex items-center justify-between mt-4 pt-4 border-t border-gray-50">
                    <div className="flex gap-4 text-sm text-gray-500">
                      {pkg.authors && pkg.authors.length > 0 && (
                        <span>{pkg.authors[0]}</span>
                      )}
                      {pkg.license && <span>{pkg.license}</span>}
                    </div>
                    <Link to={`/d/${pkg.name}`} className="text-sm font-medium text-bio-green hover:underline">
                      View Docs →
                    </Link>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-12 bg-white rounded-xl border border-dashed border-gray-300">
              <Text className="text-gray-500">No results found for "{searchQuery}".</Text>
            </div>
          )}
        </div>

        <div className="space-y-6">
          <div className="bg-bio-black text-bio-cream p-6 rounded-xl shadow-lg">
            <Heading level={3} className="text-white mb-4">Install loft</Heading>
            <div className="bg-gray-800 p-4 rounded-lg font-mono text-sm overflow-x-auto mb-4 text-green-400">
              curl -fsSL https://loft.fargone.sh/install.sh | sh
            </div>
            <Text variant="caption" className="text-gray-400">Supports Linux and macOS</Text>
          </div>

          <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
            <Heading level={3} className="mb-4">
              {searchQuery ? 'Documentation Matches' : 'Documentation'}
            </Heading>
            <ul className="space-y-4">
              {searchQuery && docResults.length > 0 ? (
                docResults.map(result => {
                  const snippet = getDocSnippet(result.item.content, result.matches);
                  return (
                    <li key={result.item.path} className="border-b border-gray-50 pb-3 last:border-0 last:pb-0">
                      <Link to={`/book/${result.item.path.replace(/\.md$/, '')}`} className="block group">
                        <div className="text-sm font-bold text-bio-black group-hover:text-bio-green transition-colors">
                          {result.item.title}
                        </div>
                        <div className="text-[11px] text-gray-500 line-clamp-2 mt-1 font-mono">
                          {highlightMatch(snippet.text, [{ key: 'text', indices: snippet.indices }], 'text')}
                        </div>
                      </Link>
                    </li>
                  );
                })
              ) : searchQuery && docResults.length === 0 ? (
                <Text className="text-xs text-gray-400 italic">No documentation matches found.</Text>
              ) : (
                <>
                  <li>
                    <Link to="/book/introduction" className="flex items-center gap-2 text-gray-700 hover:text-bio-green transition-colors text-sm">
                      <span className="w-1.5 h-1.5 rounded-full bg-bio-green"></span>
                      Book
                    </Link>
                  </li>
                  <li>
                    <Link to="/d/std" className="flex items-center gap-2 text-gray-700 hover:text-bio-green transition-colors text-sm">
                      <span className="w-1.5 h-1.5 rounded-full bg-bio-green"></span>
                      Standard Library
                    </Link>
                  </li>
                </>
              )}
              <li className="pt-2 border-t border-gray-50 mt-2">
                <Link to="/playground" className="flex items-center gap-2 text-bio-green font-bold hover:underline transition-colors text-sm">
                  <span className="w-1.5 h-1.5 rounded-full bg-bio-green"></span>
                  Try Playground!
                </Link>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </Layout>
  );
};

const App = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/auth/callback" element={<AuthCallback />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/book/*" element={<Book />} />
        <Route path="/p/:package" element={<PackageInfo />} />
        <Route path="/d/:package/*" element={<Docs />} />
        <Route path="/playground" element={<Playground />} />
        <Route path="*" element={<NotFound />} />
      </Routes>
    </BrowserRouter>
  );
};

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);
