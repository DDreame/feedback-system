import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { FeedbackWidget, FeedbackWidgetConfig } from '../src/index';

const mockInitResponse = {
  endUser: {
    id: 'end-user-123',
    projectId: 'project-123',
    deviceId: 'device-456',
    name: undefined,
  },
  conversation: {
    id: 'conv-789',
    projectId: 'project-123',
    endUserId: 'end-user-123',
    status: 'open' as const,
  },
};

// Mock WebSocket class to track instances
class MockWebSocketClass {
  static instances: MockWebSocketClass[] = [];
  readyState = WebSocket.CONNECTING;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: ((error: Event) => void) | null = null;

  constructor(public url: string) {
    MockWebSocketClass.instances.push(this);
    // Simulate async connection
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      this.onopen?.();
    }, 0);
  }

  send = vi.fn();
  close = vi.fn(() => {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  });

  // Helper to simulate connection close
  simulateClose() {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  }

  static reset() {
    MockWebSocketClass.instances = [];
  }
}

describe('FeedbackWidget WebSocket', () => {
  let mockFetch: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    MockWebSocketClass.reset();

    // Clear localStorage
    vi.stubGlobal('localStorage', {
      getItem: vi.fn().mockReturnValue(null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
    });

    // Mock fetch
    mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockInitResponse),
    });
    vi.stubGlobal('fetch', mockFetch);

    // Mock crypto.randomUUID
    vi.stubGlobal('crypto', {
      randomUUID: () => 'test-uuid-123',
    });

    // Mock WebSocket
    vi.stubGlobal('WebSocket', MockWebSocketClass);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('auto-reconnect', () => {
    it('should attempt reconnection on connection close', async () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      // Initialize widget (this will trigger WebSocket connection)
      await widget.init();

      // Get the first WebSocket instance
      const firstWs = MockWebSocketClass.instances[0];

      // Simulate connection close
      firstWs.simulateClose();

      // Wait for reconnection attempt
      await new Promise(resolve => setTimeout(resolve, 1500));

      // Should have created a new WebSocket instance (reconnect)
      expect(MockWebSocketClass.instances.length).toBe(2);

      widget.destroy();
    });

    it('should use exponential backoff for reconnection', async () => {
      vi.useFakeTimers();

      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      // First disconnect
      MockWebSocketClass.instances[0].simulateClose();

      // Fast-forward time but don't let reconnect happen yet
      vi.advanceTimersByTime(500);
      expect(MockWebSocketClass.instances.length).toBe(1);

      // Let first reconnect happen (1000ms initial delay)
      vi.advanceTimersByTime(600);
      expect(MockWebSocketClass.instances.length).toBe(2);

      // Second disconnect
      MockWebSocketClass.instances[1].simulateClose();

      // Should reconnect after ~2000ms (exponential backoff: 1000 * 2^1)
      vi.advanceTimersByTime(1500);
      expect(MockWebSocketClass.instances.length).toBe(2);

      vi.advanceTimersByTime(600);
      expect(MockWebSocketClass.instances.length).toBe(3);

      widget.destroy();
      vi.useRealTimers();
    });

    it('should limit max reconnection attempts', async () => {
      vi.useFakeTimers();

      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      // Simulate multiple connection closes beyond max attempts (5)
      for (let i = 0; i < 6; i++) {
        const ws = MockWebSocketClass.instances[MockWebSocketClass.instances.length - 1];
        ws.simulateClose();

        // Advance time beyond reconnection delay
        vi.advanceTimersByTime(35000);
      }

      // Should not exceed 6 instances (initial + 5 reconnects)
      // After 5 reconnects, it should stop trying
      expect(MockWebSocketClass.instances.length).toBeLessThanOrEqual(6);

      widget.destroy();
      vi.useRealTimers();
    });

    it('should clear reconnection timer on destroy', async () => {
      vi.useFakeTimers();

      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      // Trigger a disconnect
      MockWebSocketClass.instances[0].simulateClose();

      // Destroy widget before reconnection happens
      widget.destroy();

      // Advance time - should NOT reconnect because we destroyed
      vi.advanceTimersByTime(5000);

      // Should still be only 1 WebSocket instance
      expect(MockWebSocketClass.instances.length).toBe(1);

      vi.useRealTimers();
    });
  });
});
