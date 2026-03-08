import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { FeedbackWidget, FeedbackWidgetConfig } from '../src/index';

describe('FeedbackWidget UI', () => {
  let container: HTMLElement;
  let widget: FeedbackWidget;
  let mockFetch: ReturnType<typeof vi.fn>;

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
      status: 'open',
    },
  };

  beforeEach(() => {
    // Create a container element for testing
    container = document.createElement('div');
    container.id = 'test-container';
    document.body.appendChild(container);

    // Mock localStorage
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
    });
    vi.mocked(localStorage.getItem).mockReturnValue(null);

    // Mock fetch
    mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockInitResponse),
    });
    vi.stubGlobal('fetch', mockFetch);

    // Mock crypto.randomUUID
    vi.stubGlobal('crypto', {
      randomUUID: () => 'test-device-id',
    });

    // Mock WebSocket
    vi.stubGlobal('WebSocket', vi.fn().mockImplementation(() => ({
      send: vi.fn(),
      close: vi.fn(),
      readyState: 1, // WebSocket.OPEN
      onopen: null,
      onclose: null,
      onmessage: null,
      onerror: null,
    })));
  });

  afterEach(() => {
    if (container && container.parentNode) {
      container.parentNode.removeChild(container);
    }
    widget?.destroy();
    vi.unstubAllGlobals();
  });

  describe('mount', () => {
    it('should mount to specified container element', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();
      expect(widget).toBeDefined();
    });
  });

  describe('Shadow DOM', () => {
    it('should create shadow root for style isolation', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      // Widget should have created a shadow root
      const shadowRoot = container.shadowRoot;
      expect(shadowRoot).not.toBeNull();
    });

    it('should not leak styles to outside', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      expect(shadowRoot).not.toBeNull();

      // Check that styles are encapsulated in shadow DOM
      const style = shadowRoot?.querySelector('style');
      expect(style).not.toBeNull();
    });
  });

  describe('Floating Button', () => {
    it('should render floating button in shadow DOM', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const button = shadowRoot?.querySelector('.feedback-floating-button');
      expect(button).not.toBeNull();
    });

    it('should show floating button by default', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const button = shadowRoot?.querySelector('.feedback-floating-button') as HTMLElement;
      expect(button).not.toBeNull();
      expect(button?.style.display).not.toBe('none');
    });
  });

  describe('Chat Window Toggle', () => {
    it('should have chat window element in shadow DOM', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const chatWindow = shadowRoot?.querySelector('.feedback-chat-window');
      expect(chatWindow).not.toBeNull();
    });

    it('should toggle chat window on button click', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const button = shadowRoot?.querySelector('.feedback-floating-button') as HTMLElement;
      const chatWindow = shadowRoot?.querySelector('.feedback-chat-window') as HTMLElement;

      // Chat window should be hidden initially
      expect(chatWindow?.classList.contains('open')).toBe(false);

      // Click button to open
      button?.click();

      // Chat window should be open
      expect(chatWindow?.classList.contains('open')).toBe(true);

      // Click button again to close
      button?.click();

      // Chat window should be closed
      expect(chatWindow?.classList.contains('open')).toBe(false);
    });
  });

  describe('Chat Window Structure', () => {
    it('should have header in chat window', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const header = shadowRoot?.querySelector('.feedback-header');
      expect(header).not.toBeNull();
    });

    it('should have messages area in chat window', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const messages = shadowRoot?.querySelector('.feedback-messages');
      expect(messages).not.toBeNull();
    });

    it('should have input area in chat window', async () => {
      widget = new FeedbackWidget({
        apiKey: 'test-api-key',
        projectId: 'test-project-id',
        container,
      });

      await widget.init();

      const shadowRoot = container.shadowRoot;
      const inputArea = shadowRoot?.querySelector('.feedback-input-area');
      expect(inputArea).not.toBeNull();
    });
  });
});
