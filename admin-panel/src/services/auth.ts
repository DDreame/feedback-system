import { api, ApiError } from '../api/client';

interface Developer {
  id: string;
  email: string;
  name: string;
  created_at: string;
}

interface LoginResponse {
  access_token: string;
  refresh_token: string;
  developer: Developer;
}

export interface LoginResult {
  accessToken: string;
  refreshToken: string;
  user: {
    id: string;
    email: string;
    name: string;
  };
}

export async function login(
  email: string,
  password: string
): Promise<LoginResult> {
  try {
    const response = await api.post<LoginResponse>(
      '/api/v1/auth/login',
      { email, password }
    );

    return {
      accessToken: response.access_token,
      refreshToken: response.refresh_token,
      user: {
        id: response.developer.id,
        email: response.developer.email,
        name: response.developer.name,
      },
    };
  } catch (error) {
    if (error instanceof ApiError) {
      throw new Error(error.message);
    }
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('An unexpected error occurred');
  }
}

export async function register(
  email: string,
  password: string,
  name: string
): Promise<{ developer: Developer }> {
  try {
    return await api.post<{ developer: Developer }>('/api/v1/auth/register', {
      email,
      password,
      name,
    });
  } catch (error) {
    if (error instanceof ApiError) {
      throw new Error(error.message);
    }
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('An unexpected error occurred');
  }
}
