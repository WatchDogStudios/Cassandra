import React, { useEffect, useRef } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider, LoginPage, ProtectedRoute, useAuth } from './auth';
import { Layout } from './components/Layout';
import { Dashboard } from './components/Dashboard';
import { AgentsPage } from './components/AgentsPage';
import { BuildsPage } from './components/BuildsPage';
import { ScriptsPage } from './components/ScriptsPage';
import { SettingsPage } from './components/SettingsPage';
import { DocsPage } from './components/DocsPage';
import { OnboardingPage } from './components/OnboardingPage';
import { setAuthHandlers } from './api';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function AuthAwareApp() {
  const { accessToken, logout } = useAuth();
  const accessTokenRef = useRef(accessToken);
  
  // Update ref synchronously during render (not in useEffect)
  accessTokenRef.current = accessToken;

  useEffect(() => {
    setAuthHandlers(
      () => accessTokenRef.current, // Use ref to always get latest value
      () => {
        logout();
        window.location.href = '/login';
      }
    );
  }, [logout]); // Only need logout dependency now

  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute />}>
        <Route element={<Layout />}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/onboarding" element={<OnboardingPage />} />
          <Route path="/agents" element={<AgentsPage />} />
          <Route path="/builds" element={<BuildsPage />} />
          <Route path="/scripts" element={<ScriptsPage />} />
          <Route path="/docs" element={<DocsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
      </Route>
    </Routes>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AuthProvider>
          <AuthAwareApp />
        </AuthProvider>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
