import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import Login from '../pages/Login';
import { useAuth } from '../hooks/useAuth';

// Mock useAuth hook
vi.mock('../hooks/useAuth', () => ({
  useAuth: vi.fn(),
}));

// Mock authService
vi.mock('../services/auth', () => ({
  login: vi.fn().mockImplementation(() => Promise.resolve({})),
}));

import { login as mockLogin } from '../services/auth';

// Cast mockLogin to any to allow mock methods
const mockLoginFn = mockLogin as ReturnType<typeof vi.fn>;

describe('Login', () => {
  const mockUseAuth = useAuth as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.resetAllMocks();
    mockUseAuth.mockReturnValue({
      isAuthenticated: false,
      user: null,
      accessToken: null,
      refreshToken: null,
      login: vi.fn(),
      logout: vi.fn(),
      setUser: vi.fn(),
      setTokens: vi.fn(),
    });
  });

  const renderLogin = () => {
    return render(
      <BrowserRouter>
        <Login />
      </BrowserRouter>
    );
  };

  it('should render login form', () => {
    renderLogin();

    expect(screen.getByLabelText(/email/i)).toBeDefined();
    expect(screen.getByLabelText(/password/i)).toBeDefined();
    expect(screen.getByRole('button', { name: /sign in/i })).toBeDefined();
  });

  it('should show error when submitting empty form', async () => {
    renderLogin();

    const submitButton = screen.getByRole('button', { name: /sign in/i });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText(/email is required/i)).toBeDefined();
      expect(screen.getByText(/password is required/i)).toBeDefined();
    });
  });

  it('should call login API on valid submission', async () => {
    const mockLoginResponse = {
      accessToken: 'access-token-123',
      refreshToken: 'refresh-token-123',
      user: {
        id: '123e4567-e89b-12d3-a456-426614174000',
        email: 'test@example.com',
        name: 'Test User',
      },
    };

    mockLoginFn.mockResolvedValueOnce(mockLoginResponse);

    renderLogin();

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
    fireEvent.change(passwordInput, { target: { value: 'password123' } });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(mockLoginFn).toHaveBeenCalledWith('test@example.com', 'password123');
    });
  });

  it('should show loading state during login', async () => {
    mockLoginFn.mockImplementation(() => new Promise(() => {})); // Never resolves

    renderLogin();

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
    fireEvent.change(passwordInput, { target: { value: 'password123' } });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText(/signing in.../i)).toBeDefined();
    });
  });

  it('should display error message on login failure', async () => {
    mockLoginFn.mockRejectedValueOnce(new Error('Invalid credentials'));

    renderLogin();

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
    fireEvent.change(passwordInput, { target: { value: 'wrongpassword' } });
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText(/invalid credentials/i)).toBeDefined();
    });
  });
});
