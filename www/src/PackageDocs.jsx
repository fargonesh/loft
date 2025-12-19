import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { Button, Text, Heading } from 'botanical-ui';
import Layout from './Layout';

const PackageDocs = () => {
  const { package: packageName } = useParams();
  const [version, setVersion] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

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

  if (loading) return (
    <Layout>
      <div className="flex justify-center py-20">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
      </div>
    </Layout>
  );

  if (error) return (
    <Layout>
      <div className="p-8 text-center">
        <Text className="text-red-500 mb-4">Error: {error}</Text>
        <Link to="/">
          <Button variant="ghost">Go Back Home</Button>
        </Link>
      </div>
    </Layout>
  );

  const docsUrl = packageName === 'std' 
    ? '/stdlib/index.html'
    : `/pkg-docs/${packageName}/${version}/index.html`;

  return (
    <Layout fullWidth>
      <div className="flex flex-col h-[calc(100vh-64px)]">
        <div className="bg-bio-cream border-b border-bio-black/10 p-4 flex justify-between items-center shadow-sm z-10">
          <div className="flex items-baseline gap-3">
            <Heading level={4} serif className="m-0">{packageName}</Heading>
            <Text variant="caption" className="opacity-60">documentation</Text>
            {version && <Text variant="mono" className="text-xs opacity-50 bg-bio-black/5 px-1.5 py-0.5 rounded">v{version}</Text>}
          </div>
          {packageName !== 'std' && (
            <Link to={`/packages/${packageName}`} className="text-sm font-medium hover:text-bio-green transition-colors flex items-center gap-1">
              View Package Details <span>→</span>
            </Link>
          )}
        </div>
        <iframe 
          src={docsUrl} 
          className="w-full flex-1 border-none bg-white"
          title={`${packageName} documentation`}
        />
      </div>
    </Layout>
  );
};

export default PackageDocs;
