import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
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

  if (loading) return <div className="p-8 font-mono">Loading...</div>;
  if (error) return <div className="p-8 font-mono text-red-500">Error: {error}</div>;

  const docsUrl = packageName === 'std' 
    ? '/stdlib/index.html'
    : `/pkg-docs/${packageName}/${version}/index.html`;

  return (
    <div className="w-full h-screen flex flex-col font-mono">
      <div className="bg-bio-cream border-b-2 border-bio-black p-4 flex justify-between items-center">
        <div className="font-bold">
            {packageName} <span className="font-normal opacity-60">docs</span>
            {version && <span className="ml-2 text-xs opacity-50">v{version}</span>}
        </div>
        <a href="/" className="text-sm hover:underline">Back to Registry</a>
      </div>
      <iframe 
        src={docsUrl} 
        className="w-full grow border-none"
        title={`${packageName} documentation`}
      />
    </div>
  );
};

export default PackageDocs;
