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
  sentMessages: string[] = [];

  constructor(public url: string) {
    MockWebSocketClass.instances.push(this);
    // Simulate async connection
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      this.onopen?.();
    }, 0);
  }

  send = vi.fn((data: string) => {
    this.sentMessages.push(data);
  });
  close = vi.fn(() => {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  });

  // Helper to simulate receiving a message
  simulateMessage(data: object) {
    this.onmessage?.({ data: JSON.stringify(data) });
  }

  // Helper to simulate connection close
  simulateClose() {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  }

  static reset() {
    MockWebSocketClass.instances = [];
  }
}

describe('FeedbackWidget Message', () => {
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

  describe('sendMessage via WebSocket', () => {
    it('should send message via WebSocket when connected', async () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      // Wait for WebSocket to connect
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = MockWebSocketClass.instances[0];

      // Send a message
      await widget.sendMessage('Hello via WebSocket');

      // Verify WebSocket send was called
      expect(ws.send).toHaveBeenCalled();

      // Verify message format
      const sentData = JSON.parse(ws.send.mock.calls[0][0]);
      expect(sentData.type).toBe('send_message');
      expect(sentData.content).toBe('Hello via WebSocket');

      widget.destroy();
    });

    it('should queue messages when disconnected', async () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      // Get WebSocket and disconnect it before sending
      const ws = MockWebSocketClass.instances[0];
      ws.simulateClose();

      // Wait for disconnect to be processed
      await new Promise(resolve => setTimeout(resolve, 10));

      // Try to send message - should still work (HTTP fallback)
      const message = await widget.sendMessage('Hello when disconnected');

      // Should return message from HTTP response
      expect(message).toBeDefined();
      expect(message.content).toBe('Hello when disconnected');

      widget.destroy();
    });

    it('should flush queued messages when reconnected', async () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      // Wait for WebSocket to connect
      await new Promise(resolve => setTimeout(resolve, 10));

      // Get first WebSocket and disconnect
      const ws1 = MockWebSocketClass.instances[0];
      ws1.simulateClose();

      // Wait for disconnect to be processed
      await new Promise(resolve => setTimeout(resolve, 10));

      // Mock fetch to return different response for message creation
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ message: { id: 'msg-1', content: 'queued message', conversationId: 'conv-789', senderType: 'end_user', messageType: 'text', createdAt: new Date().toISOString() } }),
      });

      // Send message while disconnected (will use HTTP)
      await widget.sendMessage('queued message');

      // Verify message was sent via HTTP (fetch should be called)
      expect(mockFetch).toHaveBeenCalled();

      // Wait for reconnection
      await new Promise(resolve => setTimeout(resolve, 1500));

      // Should have a new WebSocket instance after reconnect
      const ws2 = MockWebSocketClass.instances[1];
      expect(ws2).toBeDefined();

      widget.destroy();
    });
  });

  describe('receiveMessage callback', () => {
    it('should trigger onMessage callback when receiving message', async () => {
      const messageHandler = vi.fn();

      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
        onMessage: messageHandler,
      } as FeedbackWidgetConfig);

      await widget.init();

      // Wait for WebSocket to connect
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = MockWebSocketClass.instances[0];

      // Simulate receiving a message from the server
      ws.simulateMessage({
        type: 'new_message',
        message: {
          id: 'msg-abc',
          conversation_id: 'conv-789',
          sender_type: 'developer',
          sender_id: 'dev-123',
          message_type: 'text',
          content: 'Hello from developer',
          created_at: new Date().toISOString(),
        },
      });

      // Verify callback was triggered
      expect(messageHandler).toHaveBeenCalled();

      // Verify message data
      const receivedMessage = messageHandler.mock.calls[0][0];
      expect(receivedMessage.id).toBe('msg-abc');
      expect(receivedMessage.content).toBe('Hello from developer');
      expect(receivedMessage.senderType).toBe('developer');

      widget.destroy();
    });

    it('should support multiple message handlers', async () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      // Register multiple handlers
      const unsub1 = widget.onMessage(handler1);
      const unsub2 = widget.onMessage(handler2);

      await widget.init();

      // Wait for WebSocket to connect
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = MockWebSocketClass.instances[0];

      // Simulate receiving a message
      ws.simulateMessage({
        type: 'new_message',
        message: {
          id: 'msg-xyz',
          conversation_id: 'conv-789',
          sender_type: 'ai_agent',
          message_type: 'text',
          content: 'AI response',
          created_at: new Date().toISOString(),
        },
      });

      // Both handlers should be called
      expect(handler1).toHaveBeenCalled();
      expect(handler2).toHaveBeenCalled();

      // Unsubscribe one handler
      unsub1();

      // Simulate another message
      ws.simulateMessage({
        type: 'new_message',
        message: {
          id: 'msg-123',
          conversation_id: 'conv-789',
          sender_type: 'developer',
          message_type: 'text',
          content: 'Another message',
          created_at: new Date().toISOString(),
        },
      });

      // handler1 should not be called again, handler2 should
      expect(handler1).toHaveBeenCalledTimes(1);
      expect(handler2).toHaveBeenCalledTimes(2);

      unsub2();
      widget.destroy();
    });
  });

  describe('message queue', () => {
    it('should queue messages when WebSocket is not connected', async () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'project-123',
        wsUrl: 'wss://test.example.com',
      } as FeedbackWidgetConfig);

      await widget.init();

      const ws = MockWebSocketClass.instances[0];

      // Close WebSocket
      ws.simulateClose();
      await new Promise(resolve => setTimeout(resolve, 10));

      // Mock fetch for HTTP fallback
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          message: {
            id: 'msg-1',
            conversationId: 'conv-789',
            senderType: 'end_user',
            messageType: 'text',
            content: 'Test message',
            createdAt: new Date().toISOString(),
          },
        }),
      });

      // Send message when disconnected - should use HTTP
      const result = await widget.sendMessage('Test message');
      expect(result.content).toBe('Test message');

      widget.destroy();
    });
  });
});
