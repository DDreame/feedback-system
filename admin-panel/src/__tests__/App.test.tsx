import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from '../App';
import { useAuthStore } from '../stores/authStore';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
    get length() { return Object.keys(store).length; },
    key: (i: number) => Object.keys(store)[i] || null,
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('App Routing', () => {
  beforeEach(() => {
    window.localStorage.clear();
    // Reset store state
    useAuthStore.getState().logout();
  });

  it('renders login page at /login', () => {
    render(
      <MemoryRouter initialEntries={['/login']}>
        <App />
      </MemoryRouter>
    );
    expect(screen.getByRole('heading', { name: 'Login' })).toBeDefined();
  });

  it('renders dashboard at root path when authenticated', () => {
    // Set authenticated state in localStorage
    window.localStorage.setItem('accessToken', 'test-token');
    window.localStorage.setItem('refreshToken', 'test-refresh-token');
    // Initialize store with stored tokens
    useAuthStore.getState().initialize();

    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );
    // Find the h1 element in the main content area (not the sidebar)
    const headings = screen.getAllByRole('heading', { name: 'Dashboard' });
    expect(headings[0]).toBeDefined();
  });

  it('renders projects page at /projects when authenticated', () => {
    window.localStorage.setItem('accessToken', 'test-token');
    window.localStorage.setItem('refreshToken', 'test-refresh-token');
    useAuthStore.getState().initialize();

    render(
      <MemoryRouter initialEntries={['/projects']}>
        <App />
      </MemoryRouter>
    );
    const headings = screen.getAllByRole('heading', { name: 'Projects' });
    expect(headings[0]).toBeDefined();
  });

  it('renders inbox page at /inbox when authenticated', () => {
    window.localStorage.setItem('accessToken', 'test-token');
    window.localStorage.setItem('refreshToken', 'test-refresh-token');
    useAuthStore.getState().initialize();

    render(
      <MemoryRouter initialEntries={['/inbox']}>
        <App />
      </MemoryRouter>
    );
    const headings = screen.getAllByRole('heading', { name: 'Inbox' });
    expect(headings[0]).toBeDefined();
  });

  it('renders settings page at /settings when authenticated', () => {
    window.localStorage.setItem('accessToken', 'test-token');
    window.localStorage.setItem('refreshToken', 'test-refresh-token');
    useAuthStore.getState().initialize();

    render(
      <MemoryRouter initialEntries={['/settings']}>
        <App />
      </MemoryRouter>
    );
    const headings = screen.getAllByRole('heading', { name: 'Settings' });
    expect(headings[0]).toBeDefined();
  });

  it('redirects to login when not authenticated', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );
    // Should show login page
    expect(screen.getByRole('heading', { name: 'Login' })).toBeDefined();
  });
});
