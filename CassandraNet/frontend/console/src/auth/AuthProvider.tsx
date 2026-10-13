import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';

interface User {
  id: string;
  email: string;
  display_name: string | null;
  tenant_id?: string | null;
  scopes: string[];
}

interface AuthContextType {
  user: User | null;
  accessToken: string | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

const ACCESS_TOKEN_REFRESH_INTERVAL = 14 * 60 * 1000; // 14 minutes

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const refreshTimerRef = useRef<number | null>(null);
  const sessionRestoreAttemptedRef = useRef(false);

  const clearRefreshTimer = useCallback(() => {
    if (refreshTimerRef.current) {
      clearTimeout(refreshTimerRef.current);
      refreshTimerRef.current = null;
    }
  }, []);

  const scheduleRefresh = useCallback(() => {
    clearRefreshTimer();
    refreshTimerRef.current = window.setTimeout(async () => {
      try {
        const response = await fetch('/api/auth/refresh', {
          method: 'POST',
          credentials: 'same-origin',
        });
        if (response.ok) {
          const data = await response.json();
          setAccessToken(data.access_token);
          scheduleRefresh();
        } else {
          // Refresh failed, clear auth state
          setUser(null);
          setAccessToken(null);
        }
      } catch {
        setUser(null);
        setAccessToken(null);
      }
    }, ACCESS_TOKEN_REFRESH_INTERVAL);
  }, [clearRefreshTimer]);

  const login = useCallback(async (email: string, password: string) => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
      credentials: 'same-origin',
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Login failed' }));
      throw new Error(error.error || 'Login failed');
    }

    const data = await response.json();
    console.log('Login successful, setting state:', data);
    setAccessToken(data.access_token);
    setUser(data.user);
    setIsLoading(false); // Ensure loading is false after login
    scheduleRefresh();
  }, [scheduleRefresh]);

  const logout = useCallback(async () => {
    clearRefreshTimer();
    try {
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'same-origin',
      });
    } catch {
      // Ignore errors on logout
    }
    setUser(null);
    setAccessToken(null);
  }, [clearRefreshTimer]);

  const refreshUser = useCallback(async () => {
    if (!accessToken) return;
    const meResponse = await fetch('/api/auth/me', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    if (meResponse.ok) {
      const userData = await meResponse.json();
      setUser(userData);
    }
  }, [accessToken]);

  // Try to restore session on mount (only if not already authenticated)
  useEffect(() => {
    // Skip restore if we already have a session (e.g., just logged in)
    if (accessToken && user) {
      console.log('Already authenticated, skipping session restore');
      setIsLoading(false);
      return;
    }

    // Skip if we already attempted restore (prevents double-run in StrictMode)
    if (sessionRestoreAttemptedRef.current) {
      console.log('Session restore already attempted, skipping');
      setIsLoading(false);
      return;
    }
    sessionRestoreAttemptedRef.current = true;

    let mounted = true;
    const restoreSession = async () => {
      console.log('Attempting to restore session...');
      try {
        const response = await fetch('/api/auth/refresh', {
          method: 'POST',
          credentials: 'same-origin',
        });

        if (response.ok && mounted) {
          const data = await response.json();
          setAccessToken(data.access_token);
          
          // Fetch user info
          const meResponse = await fetch('/api/auth/me', {
            headers: { Authorization: `Bearer ${data.access_token}` },
          });
          
          if (meResponse.ok && mounted) {
            const userData = await meResponse.json();
            setUser(userData);
            scheduleRefresh();
          }
        }
      } catch {
        // No valid session, that's fine
      } finally {
        if (mounted) {
          setIsLoading(false);
        }
      }
    };

    // Add a small delay to avoid flash, then timeout after 2s
    const timeout = setTimeout(() => {
      if (mounted) {
        setIsLoading(false);
      }
    }, 2000);

    restoreSession().finally(() => clearTimeout(timeout));

    return () => {
      mounted = false;
      clearRefreshTimer();
    };
  }, [scheduleRefresh, clearRefreshTimer]);

  return (
    <AuthContext.Provider
      value={{
        user,
        accessToken,
        isLoading,
        isAuthenticated: !!user && !!accessToken,
        login,
        logout,
        refreshUser,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
