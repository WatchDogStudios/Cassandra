import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from './AuthProvider';

export function LoginPage() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    try {
      await login(email, password);
      navigate('/', { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main className="login-page">
      <div className="login-container">
        <div className="login-header">
          <h1>CassandraNet</h1>
          <p>Sign in to the operator console</p>
        </div>

        <form onSubmit={handleSubmit} className="login-form">
          {error && (
            <div className="login-error">
              {error}
            </div>
          )}

          <div className="form-group">
            <label htmlFor="email">Email</label>
            <input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="admin@localhost"
              required
              autoComplete="email"
              autoFocus
            />
          </div>

          <div className="form-group">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter your password"
              required
              autoComplete="current-password"
            />
          </div>

          <button
            type="submit"
            className="button-primary login-button"
            disabled={isSubmitting}
          >
            {isSubmitting ? 'Signing in...' : 'Sign in'}
          </button>
        </form>

        <div className="login-footer">
          <p className="muted">
            Default credentials: admin@localhost / (CASS_ADMIN_PASSWORD env var)
          </p>
        </div>
      </div>

      <style>{`
        .login-page {
          min-height: 100vh;
          display: flex;
          align-items: center;
          justify-content: center;
          background: var(--color-bg-primary, #0a0a0a);
          padding: 1rem;
        }

        .login-container {
          width: 100%;
          max-width: 400px;
          padding: 2rem;
          background: var(--color-bg-secondary, #141414);
          border: 1px solid var(--color-border, #2a2a2a);
          border-radius: 12px;
        }

        .login-header {
          text-align: center;
          margin-bottom: 2rem;
        }

        .login-header h1 {
          margin: 0 0 0.5rem;
          font-size: 1.75rem;
          color: var(--color-text-primary, #ffffff);
        }

        .login-header p {
          margin: 0;
          color: var(--color-text-secondary, #888);
        }

        .login-form {
          display: flex;
          flex-direction: column;
          gap: 1.25rem;
        }

        .form-group {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .form-group label {
          font-size: 0.875rem;
          font-weight: 500;
          color: var(--color-text-secondary, #888);
        }

        .form-group input {
          padding: 0.75rem 1rem;
          font-size: 1rem;
          background: var(--color-bg-primary, #0a0a0a);
          border: 1px solid var(--color-border, #2a2a2a);
          border-radius: 8px;
          color: var(--color-text-primary, #ffffff);
          transition: border-color 0.2s;
        }

        .form-group input:focus {
          outline: none;
          border-color: var(--color-primary, #3b82f6);
        }

        .form-group input::placeholder {
          color: var(--color-text-tertiary, #555);
        }

        .login-button {
          margin-top: 0.5rem;
          padding: 0.875rem 1rem;
          font-size: 1rem;
          font-weight: 500;
        }

        .login-button:disabled {
          opacity: 0.7;
          cursor: not-allowed;
        }

        .login-error {
          padding: 0.75rem 1rem;
          background: rgba(239, 68, 68, 0.1);
          border: 1px solid rgba(239, 68, 68, 0.3);
          border-radius: 8px;
          color: #ef4444;
          font-size: 0.875rem;
        }

        .login-footer {
          margin-top: 1.5rem;
          text-align: center;
        }

        .login-footer p {
          font-size: 0.75rem;
        }
      `}</style>
    </main>
  );
}
