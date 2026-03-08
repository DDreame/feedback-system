import { create } from 'zustand';

export interface User {
  id: string;
  email: string;
  name: string;
}

interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
}

interface LoginParams {
  user: User;
  accessToken: string;
  refreshToken: string;
}

interface AuthActions {
  login: (params: LoginParams) => void;
  logout: () => void;
  setUser: (user: User) => void;
  setTokens: (accessToken: string, refreshToken: string) => void;
  initialize: () => void;
}

type AuthStore = AuthState & AuthActions;

// Safe localStorage helper functions
const getStorageItem = (key: string): string | null => {
  try {
    if (typeof window === 'undefined' || !window.localStorage) {
      return null;
    }
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
};

const setStorageItem = (key: string, value: string): void => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem(key, value);
    }
  } catch {
    // Ignore storage errors
  }
};

const removeStorageItem = (key: string): void => {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.removeItem(key);
    }
  } catch {
    // Ignore storage errors
  }
};

// Load tokens from localStorage on initialization
const getStoredTokens = () => {
  const accessToken = getStorageItem('accessToken');
  const refreshToken = getStorageItem('refreshToken');
  return {
    accessToken,
    refreshToken,
    isAuthenticated: !!accessToken,
  };
};

const storedState = getStoredTokens();

export const useAuthStore = create<AuthStore>()((set) => ({
  isAuthenticated: storedState.isAuthenticated,
  user: null,
  accessToken: storedState.accessToken,
  refreshToken: storedState.refreshToken,

  login: ({ user, accessToken, refreshToken }) => {
    setStorageItem('accessToken', accessToken);
    setStorageItem('refreshToken', refreshToken);
    set({
      isAuthenticated: true,
      user,
      accessToken,
      refreshToken,
    });
  },

  logout: () => {
    removeStorageItem('accessToken');
    removeStorageItem('refreshToken');
    set({
      isAuthenticated: false,
      user: null,
      accessToken: null,
      refreshToken: null,
    });
  },

  setUser: (user) => {
    set({ user });
  },

  setTokens: (accessToken, refreshToken) => {
    setStorageItem('accessToken', accessToken);
    setStorageItem('refreshToken', refreshToken);
    set({ accessToken, refreshToken });
  },

  initialize: () => {
    const tokens = getStoredTokens();
    set({
      isAuthenticated: tokens.isAuthenticated,
      accessToken: tokens.accessToken,
      refreshToken: tokens.refreshToken,
    });
  },
}));
