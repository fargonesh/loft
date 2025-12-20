import React from 'react';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { Button, Heading, Text, GridLineHorizontal } from 'botanical-ui';

const Layout = ({ children, fullWidth = false }) => {
  const navigate = useNavigate();
  const location = useLocation();
  const isLoggedIn = !!localStorage.getItem('loft_token');

  const handleLogin = () => {
    window.location.href = '/auth/github/login';
  };

  const handleLogout = () => {
    localStorage.removeItem('loft_token');
    navigate('/');
  };

  return (
    <div className="min-h-screen bg-bio-cream font-sans text-bio-black flex flex-col">
      <div className="w-full bg-white/50 backdrop-blur-sm border-b border-bio-green/10 sticky top-0 z-50">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center gap-8">
              <Link to="/" className="flex items-center gap-2 group">
                <div className="w-8 h-8 bg-bio-green rounded-lg flex items-center justify-center text-bio-cream font-serif font-bold text-xl group-hover:scale-105 transition-transform">
                  l
                </div>
                <Heading level={4} className="m-0 group-hover:text-bio-green transition-colors">loft</Heading>
              </Link>
              
              <nav className="hidden md:flex gap-6">
                <Link to="/docs/introduction.md" className="text-sm font-medium hover:text-bio-green transition-colors">Docs</Link>
                <a href="/d/std" target="_blank" rel="noreferrer" className="text-sm font-medium hover:text-bio-green transition-colors">Std Lib</a>
              </nav>
            </div>

            <div className="flex items-center gap-4">
              {isLoggedIn ? (
                <>
                  <Button onClick={() => navigate('/dashboard')} variant="ghost"size="sm">
                    Dashboard
                  </Button>
                  <Button onClick={handleLogout} variant="ghost" size="sm">
                    Logout
                  </Button>
                </>
              ) : (
                <Button onClick={handleLogin}size="sm" className='text-white px-2'>
                  Login with GitHub
                </Button>
              )}
            </div>
          </div>
        </div>
      </div>

      <main className={`flex-grow w-full ${fullWidth ? '' : 'max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8'}`}>
        {children}
      </main>

      <footer className="bg-bio-black/5 border-t border-bio-black/5 mt-auto">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div className="col-span-1 md:col-span-2">
              <div className="flex items-center gap-2 mb-4">
                <div className="w-6 h-6 bg-bio-black rounded flex items-center justify-center text-bio-cream font-serif font-bold text-sm">
                  l
                </div>
                <span className="font-bold text-lg">loft</span>
              </div>
              <Text variant="caption" className="max-w-xs">
                A modern, safe, and expressive programming language designed for building reliable software.
              </Text>
            </div>
            
            <div>
              <h4 className="font-bold mb-4 text-sm uppercase tracking-wider opacity-70">Resources</h4>
              <ul className="space-y-2 text-sm">
                <li><Link to="/docs/introduction.md" className="hover:text-bio-green">Documentation</Link></li>
                <li><a href="/d/std" className="hover:text-bio-green">Standard Library</a></li>
              </ul>
            </div>

            <div>
              <h4 className="font-bold mb-4 text-sm uppercase tracking-wider opacity-70">Community</h4>
              <ul className="space-y-2 text-sm">
                <li><a href="https://github.com/fargonesh/loft" target="_blank" rel="noreferrer" className="hover:text-bio-green">GitHub</a></li>
                <li><a href="#" className="hover:text-bio-green">Discord</a></li>
                <li><a href="#" className="hover:text-bio-green">Twitter</a></li>
              </ul>
            </div>
          </div>
          
          <div className="mt-12 pt-8 border-t border-bio-black/10 text-center opacity-60 text-sm">
            © {new Date().getFullYear()} loft Language Team. All rights reserved.
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Layout;
