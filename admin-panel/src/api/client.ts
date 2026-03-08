const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '';

interface RequestOptions extends RequestInit {
  params?: Record<string, string>;
}

class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public code?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

const getAccessToken = (): string | null => {
  if (typeof window === 'undefined') return null;
  return window.localStorage.getItem('accessToken');
};

const request = async <T>(
  endpoint: string,
  options: RequestOptions = {}
): Promise<T> => {
  const { params, ...fetchOptions } = options;

  let url = `${API_BASE_URL}${endpoint}`;

  // Add query params if provided
  if (params) {
    const searchParams = new URLSearchParams(params);
    url += `?${searchParams.toString()}`;
  }

  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...fetchOptions.headers,
  };

  // Add authorization header if token exists
  const token = getAccessToken();
  if (token) {
    (headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(url, {
    ...fetchOptions,
    headers,
  });

  if (!response.ok) {
    let errorMessage = 'An error occurred';
    let errorCode: string | undefined;

    try {
      const errorData = await response.json();
      if (errorData.error) {
        errorMessage = errorData.error.message || errorMessage;
        errorCode = errorData.error.code?.toString();
      }
    } catch {
      errorMessage = response.statusText || errorMessage;
    }

    throw new ApiError(response.status, errorMessage, errorCode);
  }

  return response.json();
};

export const api = {
  get: <T>(endpoint: string, options?: RequestOptions) =>
    request<T>(endpoint, { ...options, method: 'GET' }),

  post: <T>(endpoint: string, data?: unknown, options?: RequestOptions) =>
    request<T>(endpoint, {
      ...options,
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    }),

  put: <T>(endpoint: string, data?: unknown, options?: RequestOptions) =>
    request<T>(endpoint, {
      ...options,
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    }),

  delete: <T>(endpoint: string, options?: RequestOptions) =>
    request<T>(endpoint, { ...options, method: 'DELETE' }),
};

export { ApiError };
