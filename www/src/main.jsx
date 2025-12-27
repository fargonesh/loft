import { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';
import { 
  BrutalCard, 
  Button, 
  Input, 
  Heading, 
  Text, 
  GridLineHorizontal,
} from 'botanical-ui';
import { Link } from 'react-router-dom';

import 'botanical-ui/style.css';
import './index.css';

import AuthCallback from './AuthCallback';
import Dashboard from './Dashboard';
import Docs from './Docs';
import PackageInfo from './PackageInfo';
import PackageDocs from './PackageDocs';
import Layout from './Layout';

const Home = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [packages, setPackages] = useState([]);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    searchPackages();
  }, []);

  const searchPackages = async () => {
    setLoading(true);
    try {
      const response = await fetch('/packages');
      if (response.ok) {
        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('text/html')) {
          throw new Error('Backend returned HTML instead of JSON');
        }
        const data = await response.json();
        const filtered = data.filter(pkg => 
          pkg.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          (pkg.description && pkg.description.toLowerCase().includes(searchQuery.toLowerCase()))
        );
        setPackages(filtered);
      }
    } catch (error) {
      console.error('Failed to fetch packages:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Layout>
      <div className="text-center mb-16 pt-8">
        <Heading level={1} serif className="mb-6 text-5xl md:text-6xl text-bio-green-dark">
          Build reliable software<br/>with <span className="text-bio-green">loft</span>
        </Heading>
        <Text variant="body" className="mb-10 text-xl max-w-2xl mx-auto text-gray-600">
          A modern, safe, and expressive programming language designed for the next generation of systems.
        </Text>
        
        <div className="flex max-w-xl mx-auto gap-3 p-2 bg-white rounded-xl shadow-lg border border-gray-100">
          <Input 
            placeholder="Search packages..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 border-none focus:ring-0 text-lg"
          />
          <Button onClick={searchPackages}>Search</Button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        <div className="md:col-span-2 space-y-6">
          <div className="flex items-center justify-between mb-4">
            <Heading level={2} className="text-2xl">Popular Packages</Heading>
            <Link to="/packages" className="text-bio-green hover:underline font-medium">View all</Link>
          </div>
          
          {loading ? (
            <div className="flex justify-center py-12">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
            </div>
          ) : packages.length > 0 ? (
            <div className="grid gap-4">
              {packages.map((pkg) => (
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
              <Text className="text-gray-500">No packages found matching your search.</Text>
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
            <Heading level={3} className="mb-4">Documentation</Heading>
            <ul className="space-y-3">
              <li>
                <Link to="/docs/introduction.md" className="flex items-center gap-2 text-gray-700 hover:text-bio-green transition-colors">
                  <span className="w-1.5 h-1.5 rounded-full bg-bio-green"></span>
                  Language Guide
                </Link>
              </li>
              <li>
                <Link to="/d/std" className="flex items-center gap-2 text-gray-700 hover:text-bio-green transition-colors">
                  <span className="w-1.5 h-1.5 rounded-full bg-bio-green"></span>
                  Standard Library
                </Link>
              </li>
              <li>
                <Link to="/docs/architecture.md" className="flex items-center gap-2 text-gray-700 hover:text-bio-green transition-colors">
                  <span className="w-1.5 h-1.5 rounded-full bg-bio-green"></span>
                  Architecture
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
        <Route path="/docs/*" element={<Docs />} />
        <Route path="/p/:package" element={<PackageInfo />} />
        <Route path="/d/:package" element={<PackageDocs />} />
      </Routes>
    </BrowserRouter>
  );
};

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);
