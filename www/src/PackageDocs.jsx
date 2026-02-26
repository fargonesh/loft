import React, { useState, useEffect, useRef } from 'react';
import { useParams, Link, useNavigate, useLocation } from 'react-router-dom';
import { Text, Heading } from 'botanical-ui';
import BrutalButton from './BrutalButton';
import Layout from './Layout';

const PackageDocs = () => {
  const { package: packageName, '*': subpath } = useParams();
  const navigate = useNavigate();
  const location = useLocation();
  const [version, setVersion] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [content, setContent] = useState('');
  const contentRef = useRef(null);

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
      const fileName = subpath || 'index.html';
      const docsUrl = packageName === 'std' 
        ? `/stdlib/${fileName}`
        : `/pkg-docs/${packageName}/${version}/${fileName}`;

      fetch(docsUrl)
        .then(res => {
          if (!res.ok) throw new Error('Documentation file not found');
          return res.text();
        })
        .then(html => {
          // Extract body content or just take results
          const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
          let bodyContent = bodyMatch ? bodyMatch[1] : html;
          
          // Also extract style tags to ensure they work
          const styleMatches = [...html.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/gi)];
          const styles = styleMatches.map(m => m[0]).join('\n');
          
          setContent(styles + bodyContent);
        })
        .catch(err => {
          setError(err.message);
        });
    }
  }, [packageName, version, subpath, loading]);

  useEffect(() => {
    const handleClick = (e) => {
      const target = e.target.closest('a');
      if (target && target.getAttribute('href')) {
        const href = target.getAttribute('href');
        
        // Handle links within the documentation app
        if (href.startsWith('/d/') || (!href.startsWith('http') && !href.startsWith('/') && !href.startsWith('#'))) {
          e.preventDefault();
          
          if (href.startsWith('/d/')) {
            // Internal app link already correctly formatted
            navigate(href);
          } else {
            // Relative link - resolve based on current subpath
            const currentSubpathParts = (subpath || 'index.html').split('/');
            currentSubpathParts.pop(); // Remove current file name
            const base = currentSubpathParts.join('/');
            const absoluteHref = base ? `${base}/${href}` : href;
            navigate(`/d/${packageName}/${absoluteHref}`);
          }
        }
      }
    };

    const container = contentRef.current;
    if (container) {
      container.addEventListener('click', handleClick);
      return () => container.removeEventListener('click', handleClick);
    }
  }, [content, navigate, packageName, subpath]);

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

  return (
    <Layout fullWidth>
      <div className="min-h-[calc(100vh-64px)] bg-bio-cream flex flex-col">
        {/* Breadcrumb / Top Bar */}
        <div className="bg-white border-b border-bio-black/10 px-6 py-3 flex justify-between items-center sticky top-0 z-20">
          <div className="flex items-center gap-3">
            <Link to={`/d/${packageName}`} className="hover:text-bio-green transition-colors">
              <Heading level={4} serif className="m-0">{packageName}</Heading>
            </Link>
            <div className="h-4 w-px bg-gray-200"></div>
            {version && (
              <Text variant="mono" className="text-xs bg-bio-green/10 text-bio-green px-2 py-0.5 rounded">
                v{version}
              </Text>
            )}
            <Text variant="caption" className="opacity-40 italic">documentation</Text>
          </div>
          
          <div className="flex items-center gap-6">
            {packageName !== 'std' && (
              <Link to={`/p/${packageName}`} className="text-sm font-bold text-bio-black hover:text-bio-green transition-colors flex items-center gap-1.5">
                Package Info
                <span className="text-xs">↗</span>
              </Link>
            )}
            <Link to="/" className="text-sm font-bold text-bio-black hover:text-bio-green transition-colors">
              Registry
            </Link>
          </div>
        </div>

        {/* Documentation Content */}
        <div 
          ref={contentRef}
          className="flex-1 overflow-auto bg-white flex flex-row docs-root"
          dangerouslySetInnerHTML={{ __html: content }}
        />
      </div>
    </Layout>
  );
};

export default PackageDocs;
