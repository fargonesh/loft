import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import {
  BrutalCard,
  Heading,
  Text,
  GridLineHorizontal
} from 'botanical-ui';
import BrutalButton from './BrutalButton';
import Layout from './Layout';

const PackageInfo = () => {
  const { package: packageName } = useParams();
  const [versions, setVersions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (packageName === 'std') {
      setVersions([{
        name: 'std',
        version: 'latest',
        description: 'The loft standard library',
        authors: ['The loft Team'],
        license: 'MIT',
        repository: 'https://github.com/fargonesh/loft'
      }]);
      setLoading(false);
      return;
    }

    fetch(`/packages/${packageName}`)
      .then(res => {
        if (!res.ok) throw new Error('Package not found');
        return res.json();
      })
      .then(data => {
        // Sort by version (simple string sort for now, semver would be better)
        setVersions(data.reverse());
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
      <div className="p-8 bg-red-50 border border-red-200 rounded-lg text-red-700">
        <Heading level={3} className="mb-2">Error</Heading>
        <Text>{error}</Text>
      </div>
    </Layout>
  );

  const latest = versions[0];
  let authors = [...latest.owners, ...latest.authors].filter((v, i, a) => a.indexOf(v) == i);

  return (
    <Layout>
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <Link to="/" className="text-sm text-gray-500 hover:text-bio-green mb-4 inline-block">← Back to Packages</Link>
          <div className="flex justify-between items-start">
            <div>
              <Heading level={1} className="mb-2">{latest.name}</Heading>
              <Text className="text-xl text-gray-600">{latest.description}</Text>
            </div>
            <div className="text-right">
              <div className="text-2xl font-mono font-bold text-bio-green">v{latest.version}</div>
              <Text className="text-sm text-gray-500">Latest version</Text>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div className="md:col-span-2 space-y-8">
            <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
              <Heading level={3} className="mb-4 border-b border-gray-100 pb-2">Installation</Heading>
              <div className="bg-gray-900 text-gray-100 p-4 rounded-lg font-mono text-sm flex justify-between items-center group">
                <span>loft add {latest.name}</span>
                <button
                  className="opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white"
                  onClick={() => navigator.clipboard.writeText(`loft add ${latest.name}`)}
                >
                  Copy
                </button>
              </div>
            </div>

            <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
              <Heading level={3} className="mb-4 border-b border-gray-100 pb-2">Readme</Heading>
              <div className="prose prose-bio max-w-none">
                <Text className="italic text-gray-500">No README available for this version.</Text>
              </div>
            </div>
          </div>

          <div className="space-y-6">
            <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
              <Heading level={4} className="mb-4 text-sm uppercase tracking-wider text-gray-500">Metadata</Heading>

              <div className="space-y-4">
                <div>
                  <div className="text-xs text-gray-500 mb-1">License</div>
                  <div className="font-medium">{latest.license || 'None'}</div>
                </div>

                <div>
                  <div className="text-xs text-gray-500 mb-1">Authors</div>
                  <div className="font-medium">
                    {authors.map((v, i, a) => (
                      <>
                        <a href={`https://github.com/${v}`}>{v}</a>
                        {i !== a.length && ', '}
                      </>
                    ))}
                  </div>
                </div>

                <div>
                  <div className="text-xs text-gray-500 mb-1">Repository</div>
                  {latest.repository ? (
                    <a href={latest.repository} target="_blank" rel="noreferrer" className="text-bio-green hover:underline break-all">
                      {latest.repository.replace('https://github.com/', '')}
                    </a>
                  ) : (
                    <span className="text-gray-400">None</span>
                  )}
                </div>
              </div>
            </div>

            <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
              <Heading level={4} className="mb-4 text-sm uppercase tracking-wider text-gray-500">Links</Heading>
              <div className="space-y-4">
                <Link to={`/d/${latest.name}`} className="block w-full">
                  <BrutalButton className="w-full">Documentation</BrutalButton>
                </Link>
                {latest.repository && (
                  <a href={latest.repository} target="_blank" rel="noreferrer" className="block w-full">
                    <BrutalButton variant="outline" className="w-full">GitHub Repository</BrutalButton>
                  </a>
                )}
              </div>
            </div>

            <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
              <Heading level={4} className="mb-4 text-sm uppercase tracking-wider text-gray-500">Versions</Heading>
              <div className="space-y-2 max-h-60 overflow-y-auto">
                {versions.map(v => (
                  <div key={v.version} className="flex justify-between items-center text-sm py-1 border-b border-gray-50 last:border-0">
                    <span className="font-mono">{v.version}</span>
                    <span className="text-gray-400 text-xs">
                      {/* Date would go here if available */}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default PackageInfo;
