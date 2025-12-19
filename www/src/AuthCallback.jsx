import React, { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { Heading, Text } from 'botanical-ui';

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
    <div className="flex items-center justify-center min-h-screen bg-bio-cream">
      <div className="text-center">
        <Heading level={2}>Authenticating...</Heading>
        <Text>Please wait while we log you in.</Text>
      </div>
    </div>
  );
};

export default AuthCallback;
