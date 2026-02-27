import React, { useState, useEffect, useRef, useMemo } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { createHighlighter } from 'shiki';
import Fuse from 'fuse.js';
import { Text, Heading } from 'botanical-ui';
import BrutalButton from './BrutalButton';
import Layout from './Layout';
import loftGrammar from './loft.tmLanguage.json';

// Parse the sidebar nav sections out of the fetched HTML string.
// Returns [{ section: string, items: [{ label, href, subItems?: [{label, href}] }] }]
function parseSidebarSections(html) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const sections = [];
  doc.querySelectorAll('.nav-section').forEach(el => {
    const h3 = el.querySelector('h3');
    const section = h3 ? h3.textContent.trim() : '';
    const topList = el.querySelector(':scope > ul');
    const items = topList ? [...topList.querySelectorAll(':scope > li')].map(li => {
      const a = li.querySelector(':scope > a');
      const subList = li.querySelector('.nav-subitems');
      const subItems = subList
        ? [...subList.querySelectorAll('li > a')].map(subA => ({
          label: subA.textContent.trim(),
          href: subA.getAttribute('href') || '',
        }))
        : [];
      return {
        label: a ? a.textContent.trim() : '',
        href: a ? (a.getAttribute('href') || '') : '',
        subItems: subItems.length > 0 ? subItems : undefined,
      };
    }) : [];
    if (items.length) sections.push({ section, items });
  });
  return sections;
}

// Extract just the .content div innerHTML (falls back to full body).
function parseMainContent(html) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const content = doc.querySelector('.content');
  return content ? content.innerHTML : (doc.body ? doc.body.innerHTML : html);
}

// Extract the href of the first <link rel="stylesheet"> in the HTML.
function parseStylesheetHref(html) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const link = doc.querySelector('link[rel="stylesheet"]');
  return link ? link.getAttribute('href') : null;
}

// Parse h2 sections and h3[id] subitems from the rendered content HTML.
// Returns [{label, id, items: [{id, label}]}]
function parseContentHeadings(html) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(`<div>${html}</div>`, 'text/html');
  const sections = [];
  let current = null;
  for (const el of doc.querySelectorAll('h2, h3[id]')) {
    if (el.tagName === 'H2') {
      const id = el.id || el.textContent.trim().toLowerCase().replace(/\s+/g, '-');
      current = { label: el.textContent.trim(), id, items: [] };
      sections.push(current);
    } else if (el.tagName === 'H3' && el.id && current) {
      current.items.push({ id: el.id, label: el.textContent.trim() });
    }
  }
  return sections;
}

function scopeCSS(css, scope) {
  // Handle @media blocks recursively
  let result = '';
  // Tokenise the CSS into @media{...} blocks and plain rules
  const mediaRe = /(@media[^{]+)\{([\s\S]*?)\}\s*\}/g;
  let lastIndex = 0;
  let match;
  while ((match = mediaRe.exec(css)) !== null) {
    // Everything before this media block
    result += scopePlainRules(css.slice(lastIndex, match.index), scope);
    result += `${match[1]} { ${scopePlainRules(match[2], scope)} }`;
    lastIndex = match.index + match[0].length;
  }
  result += scopePlainRules(css.slice(lastIndex), scope);
  return result;
}

function scopePlainRules(css, scope) {
  return css.replace(/([^{}]+)\{([^{}]*)\}/g, (_, selector, rules) => {
    if (selector.trim().startsWith('@')) return `${selector} { ${rules} }`;
    const scoped = selector.split(',').flatMap(s => {
      const t = s.trim();
      if (!t || t === '*' || t === 'body' || t === 'html') return [];
      if (t === '.sidebar' || t.startsWith('.sidebar ') || t === '.content' || /^\.content\b/.test(t)) return [];
      return [`${scope} ${t}`];
    }).join(', ');
    return scoped ? `${scoped} { ${rules} }` : '';
  });
}

// Replace pre.example and pre.signature code blocks with Shiki-highlighted HTML.
async function highlightCodeBlocks(html, highlighter) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(`<div>${html}</div>`, 'text/html');

  for (const code of doc.querySelectorAll('pre.example code')) {
    try {
      // Dedent: strip common leading whitespace from all non-empty lines
      const raw = code.textContent;
      const lines = raw.split('\n');
      const minIndent = Math.min(
        ...lines.filter(l => l.trim().length > 0).map(l => l.match(/^(\s*)/)[1].length)
      );
      const dedented = lines.map(l => l.slice(minIndent)).join('\n').replace(/\n$/, '');

      const highlighted = highlighter.codeToHtml(dedented, {
        lang: 'loft',
        theme: 'one-dark-pro',
      });
      const tmp = document.createElement('div');
      tmp.innerHTML = highlighted;
      const shikiEl = tmp.firstChild;
      const wrapped = document.createElement('div');
      wrapped.innerHTML = wrapBlockHtml(shikiEl.outerHTML);
      code.closest('pre').replaceWith(wrapped.firstChild);
    } catch (_) { /* leave as-is on parse/highlight error */ }
  }

  // Highlight pre.signature code blocks (use textContent to strip any anchor HTML).
  for (const code of doc.querySelectorAll('pre.signature code')) {
    try {
      const raw = code.textContent.trim();
      if (!raw) continue;
      const inlineHtml = highlighter.codeToHtml(raw, { lang: 'loft', theme: 'one-dark-pro' });
      const tmp = document.createElement('div');
      tmp.innerHTML = inlineHtml;
      const shikiCode = tmp.querySelector('code');
      if (shikiCode) {
        code.innerHTML = shikiCode.innerHTML;
        const pre = code.closest('pre');
        pre.style.cssText = SIGNATURE_STYLE;
        pre.setAttribute('data-loft-block', '');
      }
    } catch (_) { /* leave as-is */ }
  }

  // Highlight inline <code> elements that are NOT inside a handled <pre>.
  // We only skip pre.example (already replaced above) and pre.signature
  // (already highlighted above). Any other pre — e.g. pre.return-type or
  // bare pres used as inline display by the doc generator — should have
  // their <code> children styled as inline tokens.
  for (const code of doc.querySelectorAll('code')) {
    if (code.closest('pre.signature, [data-loft-block]')) continue;
    try {
      const raw = code.textContent.trim();
      if (!raw) continue;
      const inlineHtml = highlighter.codeToHtml(raw, { lang: 'loft', theme: 'one-dark-pro' });
      const tmp = document.createElement('div');
      tmp.innerHTML = inlineHtml;
      const shikiCode = tmp.querySelector('code');
      if (shikiCode) {
        const tmp2 = document.createElement('span');
        tmp2.innerHTML = wrapInlineHtml(shikiCode.innerHTML);
        code.replaceWith(tmp2.firstChild);
      }
    } catch (_) { /* leave as-is */ }
  }

  return doc.querySelector('div').innerHTML;
}

const Docs = () => {
  const { package: packageName, '*': subpath } = useParams();
  const navigate = useNavigate();
  const [version, setVersion] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [rawHtml, setRawHtml] = useState('');
  const [scopedCSS, setScopedCSS] = useState('');
  const [highlighter, setHighlighter] = useState(null);
  const [highlightedContent, setHighlightedContent] = useState('');
  const contentRef = useRef(null);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);

  // Derived state from parsed HTML
  const sidebarSections = useMemo(() => (rawHtml ? parseSidebarSections(rawHtml) : []), [rawHtml]);
  const mainContent = useMemo(() => (rawHtml ? parseMainContent(rawHtml) : ''), [rawHtml]);

  // Flat search index built from sidebar nav items
  const searchIndex = useMemo(() => {
    const items = [];
    sidebarSections.forEach(({ section, items: navItems }) => {
      navItems.forEach(item => items.push({ ...item, section }));
    });
    return items;
  }, [sidebarSections]);

  const fuse = useMemo(() => new Fuse(searchIndex, {
    keys: ['label', 'section'],
    threshold: 0.35,
    includeMatches: true,
    minMatchCharLength: 2,
  }), [searchIndex]);

  const searchResults = useMemo(() => {
    if (!searchQuery) return [];
    return fuse.search(searchQuery).slice(0, 8);
  }, [searchQuery, fuse]);

  const highlightMatch = (text, matches, key) => {
    const match = matches?.find(m => m.key === key);
    if (!match || !match.indices || match.indices.length === 0) return text;
    const parts = [];
    let lastIndex = 0;
    const sorted = [...match.indices].sort((a, b) => a[0] - b[0]);
    sorted.forEach(([start, end], i) => {
      parts.push(text.substring(lastIndex, start));
      parts.push(<mark key={i} className="bg-bio-green/30 text-bio-green-dark font-bold rounded px-0.5">{text.substring(start, end + 1)}</mark>);
      lastIndex = end + 1;
    });
    parts.push(text.substring(lastIndex));
    return parts;
  };

  const contentHeadings = useMemo(() => {
    const html = highlightedContent || mainContent;
    return html ? parseContentHeadings(html) : [];
  }, [highlightedContent, mainContent]);

  // ⌘K / Ctrl+K keyboard shortcut to open search
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

  // Initialise Shiki once
  useEffect(() => {
    createHighlighter({
      themes: ['one-dark-pro'],
      langs: [
        { ...loftGrammar, name: 'loft', aliases: ['lf'] },
        'rust', 'bash', 'json',
      ],
    }).then(setHighlighter);
  }, []);

  // Apply syntax highlighting whenever content or highlighter changes
  useEffect(() => {
    if (!mainContent) return;
    if (!highlighter) {
      setHighlightedContent(mainContent);
      return;
    }
    highlightCodeBlocks(mainContent, highlighter).then(setHighlightedContent);
  }, [mainContent, highlighter]);

  useEffect(() => {
    if (packageName === 'std') {
      setLoading(false);
      return;
    }

    fetch(`/packages/${packageName}`)
      .then(res => {
        if (!res.ok) throw new Error('Package not found');
        return res.json();
      })
      .then(data => {
        if (data.length > 0) {
          // API returns versions in insertion order (oldest first)
          const latest = data[data.length - 1];
          setVersion(latest.version);
        } else {
          setError('No versions found');
        }
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, [packageName]);

  useEffect(() => {
    if ((packageName === 'std' || version) && !loading) {
      const fileName = subpath
        ? (subpath.endsWith('.html') ? subpath : subpath + '.html')
        : 'index.html';
      const docsUrl = packageName === 'std'
        ? `/stdlib/${fileName}`
        : `/pkg-docs/${packageName}/${version}/${fileName}`;

      fetch(docsUrl)
        .then(res => {
          if (!res.ok) throw new Error('Documentation file not found');
          return res.text();
        })
        .then(async html => {
          setRawHtml(html);

          // Fetch and scope the stylesheet so doc-specific classes render correctly
          const cssHref = parseStylesheetHref(html);
          if (cssHref) {
            const base = docsUrl.substring(0, docsUrl.lastIndexOf('/') + 1);
            const cssUrl = cssHref.startsWith('/') ? cssHref : base + cssHref;
            try {
              const cssRes = await fetch(cssUrl);
              if (cssRes.ok) {
                const cssText = await cssRes.text();
                setScopedCSS(scopeCSS(cssText, '.pkg-doc-content'));
              }
            } catch (_) {
              // CSS is optional — gracefully ignore
            }
          }
        })
        .catch(err => setError(err.message));
    }
  }, [packageName, version, subpath, loading]);

  // Intercept clicks on relative links and route them through React Router
  useEffect(() => {
    const handleClick = (e) => {
      const target = e.target.closest('a');
      if (!target) return;
      const href = target.getAttribute('href');
      if (!href) return;

      if (href.startsWith('/d/') || (!href.startsWith('http') && !href.startsWith('/') && !href.startsWith('#'))) {
        e.preventDefault();
        if (href.startsWith('/d/')) {
          navigate(href.replace(/\.html$/, ''));
        } else {
          const currentSubpathParts = (subpath || 'index.html').split('/');
          currentSubpathParts.pop();
          const base = currentSubpathParts.join('/');
          const absoluteHref = base ? `${base}/${href}` : href;
          navigate(`/d/${packageName}/${absoluteHref.replace(/\.html$/, '')}`);
        }
      }
    };

    const container = contentRef.current;
    if (container) {
      container.addEventListener('click', handleClick);
      return () => container.removeEventListener('click', handleClick);
    }
  }, [mainContent, navigate, packageName, subpath]);

  if (loading) return (
    <Layout>
      <div className="flex justify-center py-20">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
      </div>
    </Layout>
  );

  if (error) return (
    <Layout>
      <div className="p-8 text-center border-2 border-dashed border-red-200 bg-red-50 rounded-lg m-8">
        <Heading level={2} className="text-red-600 mb-2">Error Loading Docs</Heading>
        <Text className="text-red-500 mb-6">{error}</Text>
        <div className="flex justify-center gap-4">
          <Link to="/">
            <BrutalButton variant="ghost">Go Back Home</BrutalButton>
          </Link>
          <BrutalButton onClick={() => window.location.reload()}>Retry</BrutalButton>
        </div>
      </div>
    </Layout>
  );

  const currentFile = subpath
    ? (subpath.endsWith('.html') ? subpath : subpath + '.html')
    : 'index.html';

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
                  placeholder="Search docs..."
                  value={searchQuery}
                  onChange={e => { setSearchQuery(e.target.value); setSelectedIndex(0); }}
                  className="w-full text-lg border-none focus:ring-0 bg-transparent outline-none"
                />
              </div>
              <div className="max-h-[60vh] overflow-y-auto p-2">
                {searchResults.length > 0 ? (
                  searchResults.map((result, i) => {
                    const item = result.item;
                    return (
                      <button
                        key={item.href + i}
                        onClick={() => {
                          setSearchOpen(false);
                          setSearchQuery('');
                          const base = (subpath || 'index.html').split('/').slice(0, -1).join('/');
                          const resolved = base ? `${base}/${item.href}` : item.href;
                          navigate(`/d/${packageName}/${resolved.replace(/\.html$/, '')}`);
                        }}
                        className={`block w-full text-left p-4 rounded-lg hover:bg-bio-green/10 transition-colors border-2 mb-2 ${i === selectedIndex ? 'bg-bio-green/5 border-bio-green' : 'border-transparent'}`}
                      >
                        <div className="flex justify-between items-start mb-1">
                          <div className="font-bold text-bio-black text-lg">
                            {highlightMatch(item.label, result.matches, 'label')}
                          </div>
                          <div className="text-[10px] bg-bio-black/5 text-bio-black/40 px-2 py-0.5 rounded font-mono uppercase tracking-tighter">
                            {item.section || 'General'}
                          </div>
                        </div>
                      </button>
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
        <aside className="min-h-screen relative w-full md:w-64 bg-bio-offwhite border-r border-bio-black/5 p-6 md:sticky md:top-16 self-start">
          {/* Package header */}
          <div className="mb-8">
            <Link to={`/d/${packageName}`} className="hover:text-bio-green transition-colors block">
              <Heading level={4} serif className="m-0">{packageName}</Heading>
            </Link>
            <div className="flex items-center gap-2 mt-2">
              {version && (
                <Text variant="mono" className="text-xs bg-bio-green/10 text-bio-green px-2 py-0.5 rounded">
                  v{version}
                </Text>
              )}
              <Text variant="caption" className="opacity-40 italic text-xs">documentation</Text>
            </div>
          </div>

          {/* Search button */}
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

          {/* Nav sections extracted from the HTML docs */}
          <div className="space-y-8">
            {sidebarSections.map(({ section, items }) => (
              <div key={section}>
                <Text variant="mono" className="text-[11px] uppercase tracking-widest font-bold text-bio-black/40 mb-3 block">
                  {section || 'General'}
                </Text>
                <ul className="space-y-1 border-l border-bio-black/5 ml-1 pl-3">
                  {items.map(item => (
                    <li key={item.href}>
                      <button
                        onClick={() => {
                          if (item.href.startsWith('#')) {
                            const el = contentRef.current?.querySelector(item.href);
                            el?.scrollIntoView({ behavior: 'smooth' });
                          } else {
                            const base = (subpath || 'index.html').split('/').slice(0, -1).join('/');
                            const resolved = base ? `${base}/${item.href}` : item.href;
                            navigate(`/d/${packageName}/${resolved.replace(/\.html$/, '')}`);
                          }
                        }}
                        className={`text-sm block py-1.5 transition-colors w-full text-left ${currentFile === item.href || currentFile.endsWith('/' + item.href) ? 'font-bold text-bio-green -ml-3.5 pl-3.5 border-l-2 border-bio-green' : 'text-bio-black/70 hover:text-bio-black'}`}
                      >
                        {item.label}
                      </button>
                      {item.subItems && item.subItems.length > 0 && (
                        <ul className="mt-0.5 mb-1 space-y-0.5 ml-1 border-l border-bio-black/5 pl-2.5">
                          {item.subItems.map(sub => (
                            <li key={sub.href}>
                              <button
                                onClick={() => {
                                  if (sub.href.startsWith('#')) {
                                    const el = contentRef.current?.querySelector(sub.href);
                                    el?.scrollIntoView({ behavior: 'smooth' });
                                  } else {
                                    const base = (subpath || 'index.html').split('/').slice(0, -1).join('/');
                                    const resolved = base ? `${base}/${sub.href}` : sub.href;
                                    navigate(`/d/${packageName}/${resolved.replace(/\.html$/, '')}`);
                                  }
                                }}
                                className="text-xs block py-0.5 font-mono text-bio-black/50 hover:text-bio-green transition-colors w-full text-left"
                              >
                                {sub.label}
                              </button>
                            </li>
                          ))}
                        </ul>
                      )}
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>

          {/* On this page — h2/h3 anchor subitems for the current doc page */}
          {currentFile !== 'index.html' && contentHeadings.length > 0 && (
            <div className="mt-8 pt-6 border-t border-bio-black/5">
              <Text variant="mono" className="text-[11px] uppercase tracking-widest font-bold text-bio-black/40 mb-3 block">
                On this page
              </Text>
              <div className="space-y-4">
                {contentHeadings.map(section => (
                  <div key={section.id}>
                    <button
                      onClick={() => {
                        const el = contentRef.current?.querySelector('#' + CSS.escape(section.id));
                        el?.scrollIntoView({ behavior: 'smooth' });
                      }}
                      className="text-xs font-bold text-bio-black/50 uppercase tracking-wide block w-full text-left hover:text-bio-black transition-colors"
                    >
                      {section.label}
                    </button>
                    {section.items.length > 0 && (
                      <ul className="mt-1 space-y-0.5 ml-1 border-l border-bio-black/5 pl-3">
                        {section.items.map(item => (
                          <li key={item.id}>
                            <button
                              onClick={() => {
                                const el = contentRef.current?.querySelector('#' + CSS.escape(item.id));
                                el?.scrollIntoView({ behavior: 'smooth' });
                              }}
                              className="text-sm py-1 text-bio-black/60 hover:text-bio-green transition-colors font-mono w-full text-left"
                            >
                              {item.label}
                            </button>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Bottom links */}
          <div className="mt-8 pt-6 border-t border-bio-black/5 space-y-2">
            {packageName !== 'std' && (
              <Link to={`/p/${packageName}`} className="text-sm font-bold text-bio-black hover:text-bio-green transition-colors flex items-center gap-1.5">
                Package Info <span className="text-xs">↗</span>
              </Link>
            )}
            <Link to="/" className="text-sm text-bio-black/60 hover:text-bio-black transition-colors block">
              ← Registry
            </Link>
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 p-8 md:p-16 bg-white overflow-y-auto">
          <div className="max-w-3xl mx-auto">
            {!mainContent ? (
              <div className="flex justify-center py-20">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
              </div>
            ) : (
              <>
                {scopedCSS && <style>{scopedCSS}</style>}
                <article
                  ref={contentRef}
                  className="pkg-doc-content prose prose-bio max-w-none prose-headings:font-serif prose-headings:font-bold prose-h1:text-4xl prose-h2:text-2xl prose-a:text-bio-green prose-code:text-bio-green-dark prose-pre:bg-bio-black prose-pre:shadow-lg prose-pre:border-2 prose-pre:border-bio-black"
                  dangerouslySetInnerHTML={{ __html: highlightedContent || mainContent }}
                />
              </>
            )}

            <div className="mt-24 pt-8 border-t border-gray-100 flex justify-between items-center">
              <BrutalButton variant="ghost" size="sm" onClick={() => navigate('/')}>← Back to Home</BrutalButton>
              <Text variant="caption" className="text-gray-400 font-mono text-xs">LOFT LANGUAGE v0.1.0</Text>
            </div>
          </div>
        </main>

      </div>
    </Layout>
  );
};

export default Docs;