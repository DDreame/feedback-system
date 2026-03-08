/**
 * HTTP Client for Feedback System Web SDK
 * Provides a wrapper around fetch with auth headers, timeout, and error handling.
 */

export interface HttpClientConfig {
  /** The API key for authentication (required) */
  apiKey: string;
  /** Base URL for API requests (required) */
  baseUrl: string;
  /** Request timeout in ms (default: 30000ms) */
  timeout?: number;
  /** Enable debug logging (default: false) */
  debug?: boolean;
}

export interface HttpClientResponse<T = unknown> {
  data: T;
  status: number;
  ok: boolean;
}

/**
 * HttpClient - Wrapper around fetch with authentication and error handling
 */
export class HttpClient {
  private apiKey: string;
  private baseUrl: string;
  private timeout: number;
  private debug: boolean;

  constructor(config: HttpClientConfig) {
    if (!config.apiKey || config.apiKey.trim() === '') {
      throw new Error('apiKey is required');
    }
    if (!config.baseUrl || config.baseUrl.trim() === '') {
      throw new Error('baseUrl is required');
    }

    this.apiKey = config.apiKey;
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.timeout = config.timeout ?? 30000;
    this.debug = config.debug ?? false;
  }

  /**
   * Make a GET request
   */
  async get<T = unknown>(path: string): Promise<T> {
    return this.request<T>('GET', path);
  }

  /**
   * Make a POST request
   */
  async post<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('POST', path, body);
  }

  /**
   * Make a PUT request
   */
  async put<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PUT', path, body);
  }

  /**
   * Make a DELETE request
   */
  async delete<T = unknown>(path: string): Promise<T> {
    return this.request<T>('DELETE', path);
  }

  /**
   * Core request method
   */
  private async request<T = unknown>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const url = `${this.baseUrl}${path.startsWith('/') ? path : `/${path}`}`;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'X-API-Key': this.apiKey,
    };

    const requestOptions: RequestInit = {
      method,
      headers,
    };

    if (body !== undefined) {
      requestOptions.body = JSON.stringify(body);
    }

    if (this.debug) {
      console.log(`[HttpClient] ${method} ${url}`, { body });
    }

    // Create abort controller for timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);
    requestOptions.signal = controller.signal;

    try {
      const response = await fetch(url, requestOptions);
      clearTimeout(timeoutId);

      // Handle non-OK responses
      if (!response.ok) {
        let errorMessage = `HTTP ${response.status}`;

        try {
          const errorData = await response.json();
          errorMessage = errorData.error || errorData.message || errorMessage;
        } catch {
          // Response may not be JSON
        }

        throw new Error(errorMessage);
      }

      // Handle empty responses (e.g., 204 No Content)
      if (response.status === 204) {
        return {} as T;
      }

      const data = await response.json();

      if (this.debug) {
        console.log(`[HttpClient] Response:`, data);
      }

      return data as T;
    } catch (error) {
      clearTimeout(timeoutId);

      if (this.debug) {
        console.error(`[HttpClient] Error:`, error);
      }

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new Error(`Request timeout after ${this.timeout}ms`);
        }
        throw error;
      }

      throw new Error('Unknown error occurred');
    }
  }
}

export default HttpClient;
