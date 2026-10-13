import React, { useMemo } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { useAuth } from '../auth';

function IconDashboard() {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path d="M4 13.5h7V4H4v9.5ZM13 20h7V11h-7v9ZM4 20h7v-4.5H4V20ZM13 9h7V4h-7v5Z" fill="currentColor" opacity="0.92" />
    </svg>
  );
}

function IconAgents() {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path d="M7 4h10a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-4l-3 3-3-3H7a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" stroke="currentColor" strokeWidth="1.8" opacity="0.92" />
      <path d="M9 9h6M9 12h4" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" opacity="0.92" />
    </svg>
  );
}

function IconBuilds() {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path d="M7.5 7.5 12 5l4.5 2.5L12 10 7.5 7.5Z" stroke="currentColor" strokeWidth="1.8" />
      <path d="M7.5 7.5V16.5L12 19l4.5-2.5V7.5" stroke="currentColor" strokeWidth="1.8" />
      <path d="M12 10v9" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
    </svg>
  );
}

function IconScripts() {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path d="M7 4h7l3 3v13a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" stroke="currentColor" strokeWidth="1.8" />
      <path d="M14 4v3h3" stroke="currentColor" strokeWidth="1.8" />
      <path d="M8.5 12.5 7 14l1.5 1.5M15.5 12.5 17 14l-1.5 1.5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M12.2 12.2 11.8 15.8" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
    </svg>
  );
}

function IconSettings() {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path
        d="M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z"
        stroke="currentColor"
        strokeWidth="1.8"
      />
      <path
        d="M19.4 12a7.9 7.9 0 0 0-.1-1l2-1.5-2-3.4-2.4 1a7.6 7.6 0 0 0-1.7-1l-.4-2.5H10l-.4 2.5a7.6 7.6 0 0 0-1.7 1l-2.4-1-2 3.4 2 1.5a8.1 8.1 0 0 0 0 2l-2 1.5 2 3.4 2.4-1c.5.4 1.1.7 1.7 1l.4 2.5h4.8l.4-2.5c.6-.3 1.2-.6 1.7-1l2.4 1 2-3.4-2-1.5c.1-.3.1-.7.1-1Z"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinejoin="round"
        opacity="0.92"
      />
    </svg>
  );
}

function IconDocs() {
  return (
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path d="M7 4h9l3 3v13a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" stroke="currentColor" strokeWidth="1.8" />
      <path d="M16 4v3h3" stroke="currentColor" strokeWidth="1.8" />
      <path d="M8.5 11h7M8.5 14h7M8.5 17h4" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" opacity="0.92" />
    </svg>
  );
}

const navItems = [
  { to: '/', label: 'Dashboard', icon: <IconDashboard /> },
  { to: '/agents', label: 'Agents', icon: <IconAgents /> },
  { to: '/builds', label: 'Builds', icon: <IconBuilds /> },
  { to: '/scripts', label: 'Scripts', icon: <IconScripts /> },
  { to: '/docs', label: 'Docs', icon: <IconDocs /> },
  { to: '/settings', label: 'Settings', icon: <IconSettings /> },
];

export function Layout() {
  const { user, logout } = useAuth();

  const canSeeDocs = !!user?.scopes?.includes('docs:read');

  const avatarText = useMemo(() => {
    const email = user?.email?.trim();
    if (!email) return 'CN';
    const first = email[0]?.toUpperCase() ?? 'C';
    const second = email.includes('@') ? email.split('@')[0]?.[1]?.toUpperCase() : '';
    return `${first}${second || ''}`.slice(0, 2);
  }, [user?.email]);

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <span className="brand-icon" aria-hidden="true">CN</span>
          <span>
            <span className="brand-text">CassandraNet</span>
            <span className="brand-subtext">Console</span>
          </span>
        </div>
        
        <nav className="sidebar-nav">
          {navItems
            .filter(item => (item.to === '/docs' ? canSeeDocs : true))
            .map(item => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) => `nav-item ${isActive ? 'nav-item-active' : ''}`}
              end={item.to === '/'}
            >
              <span className="nav-icon">{item.icon}</span>
              <span className="nav-label">{item.label}</span>
            </NavLink>
          ))}
        </nav>

        <div className="sidebar-footer">
          {user && (
            <div className="user-badge">
              <span className="user-avatar" aria-hidden="true">{avatarText}</span>
              <span className="user-email">{user.email}</span>
            </div>
          )}
          <button className="button-secondary sidebar-logout" onClick={logout}>
            Sign out
          </button>
        </div>
      </aside>

      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
