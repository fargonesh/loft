import React, { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { Heading, Text } from 'botanical-ui';
import Layout from './Layout';

const AuthCallback = () => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  useEffect(() => {
    const token = searchParams.get('token');
    if (token) {
      localStorage.setItem('loft_token', token);
      navigate('/dashboard');
    } else {
      navigate('/');
    }
  }, [searchParams, navigate]);

  return (
    <Layout>
      <div className="flex items-center justify-center h-[calc(100vh-200px)]">
        <div className="text-center">
          <Heading level={2} className="mb-4">Authenticating...</Heading>
          <Text>Please wait while we log you in.</Text>
          <div className="mt-8 flex justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-bio-green"></div>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default AuthCallback;
