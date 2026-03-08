import { describe, it, expect, vi, beforeEach } from 'vitest';
import { HttpClient } from '../src/http';

describe('HttpClient', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);
  });

  describe('constructor', () => {
    it('should create client with apiKey', () => {
      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });
      expect(client).toBeDefined();
    });

    it('should use default timeout of 30000ms', () => {
      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });
      expect(client).toBeDefined();
    });

    it('should accept custom timeout', () => {
      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
        timeout: 5000,
      });
      expect(client).toBeDefined();
    });

    it('should accept debug option', () => {
      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
        debug: true,
      });
      expect(client).toBeDefined();
    });
  });

  describe('GET requests', () => {
    it('should include X-API-Key header in GET requests', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ success: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await client.get('/test');

      expect(fetchMock).toHaveBeenCalledWith(
        'https://api.example.com/test',
        expect.objectContaining({
          method: 'GET',
          headers: expect.objectContaining({
            'X-API-Key': 'test-api-key',
          }),
        })
      );
    });

    it('should include Content-Type header in GET requests', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ success: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await client.get('/test');

      expect(fetchMock).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
          }),
        })
      );
    });
  });

  describe('POST requests', () => {
    it('should include X-API-Key header in POST requests', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ success: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await client.post('/test', { foo: 'bar' });

      expect(fetchMock).toHaveBeenCalledWith(
        'https://api.example.com/test',
        expect.objectContaining({
          method: 'POST',
          headers: expect.objectContaining({
            'X-API-Key': 'test-api-key',
          }),
        })
      );
    });

    it('should stringify body as JSON', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ success: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await client.post('/test', { foo: 'bar', count: 42 });

      expect(fetchMock).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          body: JSON.stringify({ foo: 'bar', count: 42 }),
        })
      );
    });
  });

  describe('error handling', () => {
    it('should throw error on 4xx responses', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'Not Found' }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await expect(client.get('/not-found')).rejects.toThrow('Not Found');
    });

    it('should throw error on 5xx responses', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'Internal Server Error' }), {
          status: 500,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await expect(client.get('/server-error')).rejects.toThrow('Internal Server Error');
    });

    it('should include error message from response body', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'Custom error message' }), {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      await expect(client.get('/bad-request')).rejects.toThrow('Custom error message');
    });
  });

  describe('timeout handling', () => {
    it('should throw error on timeout', async () => {
      // Create a fetch that never resolves
      const neverResolvingFetch = vi.fn(() => new Promise<Response>((_, reject) => {
        setTimeout(() => reject(new Error('Timeout')), 100);
      }));
      vi.stubGlobal('fetch', neverResolvingFetch);

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
        timeout: 50,
      });

      await expect(client.get('/slow')).rejects.toThrow();
    });
  });

  describe('response parsing', () => {
    it('should parse JSON response', async () => {
      const responseData = { success: true, data: { id: '123' } };
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(responseData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      const result = await client.get('/test');
      expect(result).toEqual(responseData);
    });

    it('should return empty object for empty response', async () => {
      fetchMock.mockResolvedValueOnce(
        new Response(null, {
          status: 204,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
      });

      const result = await client.delete('/test');
      expect(result).toEqual({});
    });
  });

  describe('debug logging', () => {
    it('should log requests when debug is enabled', async () => {
      const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ success: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      const client = new HttpClient({
        apiKey: 'test-api-key',
        baseUrl: 'https://api.example.com',
        debug: true,
      });

      await client.get('/test');

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });
});
