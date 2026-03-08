import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
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

describe('authStore', () => {
  beforeEach(() => {
    // Reset store state before each test
    useAuthStore.getState().logout();
    window.localStorage.clear();
  });

  it('should have initial state as not authenticated', () => {
    const { result } = renderHook(() => useAuthStore());
    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBe(null);
    expect(result.current.accessToken).toBe(null);
    expect(result.current.refreshToken).toBe(null);
  });

  it('should update state after login', () => {
    const { result } = renderHook(() => useAuthStore());

    const mockUser = {
      id: 'user-123',
      email: 'test@example.com',
      name: 'Test User',
    };

    act(() => {
      result.current.login({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });
    });

    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.user).toEqual(mockUser);
    expect(result.current.accessToken).toBe('access-token-123');
    expect(result.current.refreshToken).toBe('refresh-token-123');
  });

  it('should clear state after logout', () => {
    const { result } = renderHook(() => useAuthStore());

    const mockUser = {
      id: 'user-123',
      email: 'test@example.com',
      name: 'Test User',
    };

    act(() => {
      result.current.login({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });
    });

    act(() => {
      result.current.logout();
    });

    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBe(null);
    expect(result.current.accessToken).toBe(null);
    expect(result.current.refreshToken).toBe(null);
  });

  it('should persist tokens to localStorage', () => {
    const { result } = renderHook(() => useAuthStore());

    const mockUser = {
      id: 'user-123',
      email: 'test@example.com',
      name: 'Test User',
    };

    act(() => {
      result.current.login({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });
    });

    expect(localStorage.getItem('accessToken')).toBe('access-token-123');
    expect(localStorage.getItem('refreshToken')).toBe('refresh-token-123');
  });

  it('should have accessToken in localStorage after login', () => {
    const { result } = renderHook(() => useAuthStore());

    const mockUser = {
      id: 'user-123',
      email: 'test@example.com',
      name: 'Test User',
    };

    act(() => {
      result.current.login({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });
    });

    // Verify tokens are stored in localStorage
    expect(window.localStorage.getItem('accessToken')).toBe('access-token-123');
    expect(window.localStorage.getItem('refreshToken')).toBe('refresh-token-123');
  });

  it('should update user information', () => {
    const { result } = renderHook(() => useAuthStore());

    const mockUser = {
      id: 'user-123',
      email: 'test@example.com',
      name: 'Test User',
    };

    act(() => {
      result.current.login({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });
    });

    const updatedUser = {
      id: 'user-123',
      email: 'updated@example.com',
      name: 'Updated User',
    };

    act(() => {
      result.current.setUser(updatedUser);
    });

    expect(result.current.user).toEqual(updatedUser);
  });
});
