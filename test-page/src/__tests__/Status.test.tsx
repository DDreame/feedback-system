import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, cleanup } from '@testing-library/react';
import Status from '../pages/Status';

// Mock fetch
vi.stubGlobal('fetch', vi.fn());

// Mock import.meta.env
vi.mock('import.meta.env', () => ({
  env: {
    VITE_API_KEY: '',
    VITE_PROJECT_ID: '',
    VITE_API_URL: 'http://localhost:3000',
  },
}));

describe('Status', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('renders the status page title', async () => {
    // Mock successful health check
    (fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'ok', timestamp: new Date().toISOString() }),
    });

    render(<Status />);

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'System Status' })).toBeInTheDocument();
    });
  });

  it('shows backend health status', async () => {
    (fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'ok', timestamp: new Date().toISOString() }),
    });

    render(<Status />);

    await waitFor(() => {
      expect(screen.getByText(/backend/i)).toBeInTheDocument();
    });
  });

  it('has refresh button', async () => {
    (fetch as any).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'ok', timestamp: new Date().toISOString() }),
    });

    render(<Status />);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /refresh/i })).toBeInTheDocument();
    });
  });
});
