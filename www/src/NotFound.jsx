import React from 'react';
import { useNavigate } from 'react-router-dom';
import { BrutalCard, Heading, Text } from 'botanical-ui';
import Layout from './Layout';
import BrutalButton from './BrutalButton';

const NotFound = () => {
  const navigate = useNavigate();

  return (
    <Layout>
      <div className="max-w-2xl mx-auto py-20 px-4 text-center">
        <BrutalCard className="p-12 bg-white">
          <div className="mb-8">
            <div className="w-24 h-24 bg-bio-green/10 rounded-full flex items-center justify-center mx-auto mb-6">
              <span className="text-6xl">🌱</span>
            </div>
            <Heading level={1} className="text-6xl mb-2">404</Heading>
            <Heading level={3} className="text-bio-green mb-6">Page Not Found</Heading>
            <Text className="text-lg text-gray-600 mb-8">
              It looks like this branch of the project hasn't grown yet, or was pruned away. 
              Let's head back to more familiar ground.
            </Text>
          </div>
          
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <BrutalButton 
              onClick={() => navigate('/')} 
              className="px-8"
            >
              Back to Home
            </BrutalButton>
            <BrutalButton 
              onClick={() => navigate('/book/introduction.md')} 
              variant="outline"
              className="px-8"
            >
              View Documentation
            </BrutalButton>
          </div>
        </BrutalCard>
      </div>
    </Layout>
  );
};

export default NotFound;
