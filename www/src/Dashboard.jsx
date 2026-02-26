import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { BrutalCard, Heading, Text, Input } from 'botanical-ui';
import BrutalButton from './BrutalButton';
import Layout from './Layout';

const Dashboard = () => {
  const [user, setUser] = useState(null);
  const [tokens, setTokens] = useState([]);
  const [newTokenName, setNewTokenName] = useState('');
  const [createdToken, setCreatedToken] = useState(null);
  const navigate = useNavigate();

  useEffect(() => {
    const token = localStorage.getItem('loft_token');
    if (!token) {
      navigate('/');
      return;
    }

    // Fetch user info
    fetch('/auth/me', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    .then(res => {
      if (!res.ok) throw new Error('Unauthorized');
      return res.json();
    })
    .then(data => setUser(data))
    .catch(() => {
      localStorage.removeItem('loft_token');
      navigate('/');
    });

    // Fetch tokens
    fetchTokens();
  }, [navigate]);

  const fetchTokens = () => {
    const token = localStorage.getItem('loft_token');
    fetch('/tokens', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    .then(res => res.json())
    .then(data => setTokens(data));
  };

  const createToken = async () => {
    if (!newTokenName) return;
    const token = localStorage.getItem('loft_token');
    
    const res = await fetch('/tokens', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ name: newTokenName })
    });

    if (res.ok) {
      const data = await res.json();
      setCreatedToken(data.token);
      setNewTokenName('');
      fetchTokens();
    }
  };

  const revokeToken = async (id) => {
    const token = localStorage.getItem('loft_token');
    await fetch(`/tokens/${id}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${token}` }
    });
    fetchTokens();
  };

  if (!user) return (
    <Layout>
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
      </div>
    </Layout>
  );

  return (
    <Layout>
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <Heading level={1} className="mb-2">Your Dashboard</Heading>
          <Text className="text-gray-600">Take a look at your account and manage your API tokens here.</Text>
        </div>

        <div className="grid gap-8">
          <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
            <div className="flex items-center gap-6">
              {user.avatar_url ? (
                <img src={user.avatar_url} alt={user.username} className="w-20 h-20 rounded-full border-4 border-bio-cream shadow-sm" />
              ) : (
                <div className="w-20 h-20 rounded-full bg-bio-green text-white flex items-center justify-center text-2xl font-bold">
                  {user.username[0].toUpperCase()}
                </div>
              )}
              <div>
                <Heading level={3} className="mb-1">{user.username}</Heading>
                <Text className="text-gray-500 text-sm">GitHub ID: {user.github_id}</Text>
              </div>
            </div>
          </div>

          <div>
            <div className="flex justify-between items-end mb-4">
              <div>
                <Heading level={2} className="text-xl mb-1">API Tokens</Heading>
                <Text className="text-sm text-gray-600">Use these tokens to publish packages from the CLI.</Text>
              </div>
            </div>

            <div className="bg-white p-6 rounded-xl border border-gray-100 shadow-sm">
              <div className="flex gap-3 mb-8">
                <Input 
                  placeholder="Token Name (e.g. CI/CD)" 
                  value={newTokenName}
                  onChange={(e) => setNewTokenName(e.target.value)}
                  className="flex-1 py-2"
                />
                <BrutalButton onClick={createToken}>Create Token</BrutalButton>
              </div>

              {createdToken && (
                <div className="mb-8 p-4 bg-green-50 border border-green-200 rounded-lg">
                  <div className="flex items-center gap-2 mb-2 text-green-800 font-medium">
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path></svg>
                    New Token Created
                  </div>
                  <Text className="text-sm text-green-700 mb-3">Copy this token now. You won't be able to see it again.</Text>
                  <div className="relative">
                    <code className="block p-3 bg-white border border-green-200 rounded font-mono text-sm break-all text-green-900">
                      {createdToken}
                    </code>
                  </div>
                </div>
              )}

              <div className="space-y-3">
                {tokens.length > 0 ? (
                  tokens.map(token => (
                    <div key={token.id} className="flex justify-between items-center p-4 border border-gray-100 rounded-lg hover:bg-gray-50 transition-colors">
                      <div>
                        <Text className="font-medium text-bio-black">{token.name}</Text>
                        <Text className="text-xs text-gray-500">Created on {new Date(token.created_at).toLocaleDateString()}</Text>
                      </div>
                      <BrutalButton onClick={() => revokeToken(token.id)} variant="danger" size="sm">Revoke</BrutalButton>
                    </div>
                  ))
                ) : (
                  <div className="text-center py-8 text-gray-500 bg-gray-50 rounded-lg border border-dashed border-gray-200">
                    No tokens created yet.
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default Dashboard;
