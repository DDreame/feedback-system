import { describe, it, expect, vi, beforeEach } from 'vitest';
import { login } from '../services/auth';

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

// Mock fetch globally
const fetchMock = vi.fn();
Object.defineProperty(window, 'fetch', {
  value: fetchMock,
});

describe('authService', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  describe('login', () => {
    it('should call login API and return tokens and user', async () => {
      const mockResponse = {
        access_token: 'access-token-123',
        refresh_token: 'refresh-token-123',
        developer: {
          id: '123e4567-e89b-12d3-a456-426614174000',
          email: 'test@example.com',
          name: 'Test User',
          created_at: '2024-01-01T00:00:00Z',
        },
      };

      fetchMock.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      const result = await login('test@example.com', 'password123');

      expect(fetchMock).toHaveBeenCalledWith(
        '/api/v1/auth/login',
        expect.objectContaining({
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'test@example.com',
            password: 'password123',
          }),
        })
      );

      expect(result).toEqual({
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
        user: {
          id: '123e4567-e89b-12d3-a456-426614174000',
          email: 'test@example.com',
          name: 'Test User',
        },
      });
    });

    it('should throw error when login fails', async () => {
      fetchMock.mockResolvedValue({
        ok: false,
        json: () => Promise.resolve({ error: { message: 'Invalid credentials' } }),
      });

      await expect(login('test@example.com', 'wrongpassword')).rejects.toThrow('Invalid credentials');
    });

    it('should throw error when network error occurs', async () => {
      fetchMock.mockRejectedValue(new Error('Network error'));

      await expect(login('test@example.com', 'password123')).rejects.toThrow('Network error');
    });
  });
});
