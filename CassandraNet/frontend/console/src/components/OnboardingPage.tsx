import React, { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { postJson } from '../api';
import { useAuth } from '../auth';

interface OnboardOrgResponse {
  tenant: {
    id: string;
    name: string;
    slug: string;
    plan: string;
  };
  user_id: string;
}

function slugify(input: string): string {
  return input
    .trim()
    .toLowerCase()
    .replace(/[\s_]+/g, '-')
    .replace(/[^a-z0-9-]/g, '')
    .replace(/-+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 63);
}

export function OnboardingPage() {
  const { user, refreshUser } = useAuth();
  const navigate = useNavigate();

  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [plan, setPlan] = useState<'free' | 'pro' | 'enterprise'>('free');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const autoSlug = useMemo(() => slugify(name), [name]);

  const effectiveSlug = slug.trim() ? slugify(slug) : autoSlug;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    try {
      await postJson<OnboardOrgResponse>('/api/onboarding/org', {
        name,
        slug: effectiveSlug,
        plan,
      });

      await refreshUser();
      navigate('/', { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create organization');
    } finally {
      setIsSubmitting(false);
    }
  };

  if (user?.tenant_id) {
    return (
      <div className="page-container">
        <div className="card">
          <h1>Organization already set</h1>
          <p className="muted">This account is already attached to a tenant.</p>
          <button className="button-primary" onClick={() => navigate('/', { replace: true })}>
            Go to dashboard
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <header className="page-header">
        <div>
          <h1>Set up your organization</h1>
          <p>Create your org (name + slug + plan) to start using CassandraNet.</p>
        </div>
      </header>

      <div className="card" style={{ maxWidth: 560 }}>
        <form onSubmit={handleSubmit} className="form">
          {error && <div className="alert alert-error">{error}</div>}

          <div className="form-group">
            <label htmlFor="org-name">Organization name</label>
            <input
              id="org-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Acme Corp"
              required
              autoFocus
            />
          </div>

          <div className="form-group">
            <label htmlFor="org-slug">Slug</label>
            <input
              id="org-slug"
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              placeholder={autoSlug || 'acme-corp'}
            />
            <p className="muted" style={{ marginTop: 6 }}>
              Used in URLs. Allowed: lowercase letters, digits, and hyphens.
              <br />
              Preview: <code>{effectiveSlug || '(enter a name)'}</code>
            </p>
          </div>

          <div className="form-group">
            <label htmlFor="org-plan">Plan</label>
            <select
              id="org-plan"
              value={plan}
              onChange={(e) => setPlan(e.target.value as any)}
            >
              <option value="free">Free</option>
              <option value="pro">Pro</option>
              <option value="enterprise">Enterprise</option>
            </select>
          </div>

          <div style={{ display: 'flex', gap: 12 }}>
            <button className="button-primary" type="submit" disabled={isSubmitting}>
              {isSubmitting ? 'Creating...' : 'Create organization'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
