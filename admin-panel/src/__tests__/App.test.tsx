import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from '../App';

describe('App Routing', () => {
  it('renders login page at /login', () => {
    render(
      <MemoryRouter initialEntries={['/login']}>
        <App />
      </MemoryRouter>
    );
    expect(screen.getByRole('heading', { name: 'Login' })).toBeDefined();
  });

  it('renders dashboard at root path', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );
    // Find the h1 element in the main content area (not the sidebar)
    const headings = screen.getAllByRole('heading', { name: 'Dashboard' });
    expect(headings[0]).toBeDefined();
  });

  it('renders projects page at /projects', () => {
    render(
      <MemoryRouter initialEntries={['/projects']}>
        <App />
      </MemoryRouter>
    );
    const headings = screen.getAllByRole('heading', { name: 'Projects' });
    expect(headings[0]).toBeDefined();
  });

  it('renders inbox page at /inbox', () => {
    render(
      <MemoryRouter initialEntries={['/inbox']}>
        <App />
      </MemoryRouter>
    );
    const headings = screen.getAllByRole('heading', { name: 'Inbox' });
    expect(headings[0]).toBeDefined();
  });

  it('renders settings page at /settings', () => {
    render(
      <MemoryRouter initialEntries={['/settings']}>
        <App />
      </MemoryRouter>
    );
    const headings = screen.getAllByRole('heading', { name: 'Settings' });
    expect(headings[0]).toBeDefined();
  });
});
