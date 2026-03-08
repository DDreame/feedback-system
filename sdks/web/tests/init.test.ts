import { describe, it, expect, beforeEach, vi } from 'vitest';
import { FeedbackWidget, FeedbackWidgetConfig } from '../src/index';

describe('FeedbackWidget init', () => {
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
    vi.clearAllMocks();
    // Clear localStorage
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(),
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
    vi.stubGlobal('WebSocket', vi.fn().mockImplementation(() => ({
      onopen: null,
      onclose: null,
      onmessage: null,
      onerror: null,
      close: vi.fn(),
      send: vi.fn(),
      readyState: 1, // OPEN
    })));
  });

  it('should generate device_id if not provided in localStorage', async () => {
    // Arrange: No existing session in localStorage
    vi.mocked(localStorage.getItem).mockReturnValue(null);

    widget = new FeedbackWidget({
      apiKey: 'test-key',
      projectId: 'project-123',
      apiUrl: 'http://localhost:3000',
    });

    await widget.init();

    // Verify fetch was called with device_id in request body
    expect(mockFetch).toHaveBeenCalled();
    const fetchCall = mockFetch.mock.calls[0];
    const requestBody = JSON.parse(fetchCall[1].body as string);
    expect(requestBody).toHaveProperty('device_id');
    expect(requestBody.device_id).toBe('test-uuid-123');
  });

  it('should restore session from localStorage if exists', async () => {
    // Arrange: Existing session in localStorage
    const existingSession = {
      projectId: 'project-123',
      deviceId: 'existing-device-id',
      endUserId: 'end-user-123',
      conversationId: 'conv-789',
    };
    vi.mocked(localStorage.getItem).mockReturnValue(JSON.stringify(existingSession));

    widget = new FeedbackWidget({
      apiKey: 'test-key',
      projectId: 'project-123',
      apiUrl: 'http://localhost:3000',
    });

    await widget.init();

    // Verify that existing session is used (no new fetch needed)
    const endUser = widget.getEndUser();
    const conversation = widget.getConversation();

    expect(endUser?.id).toBe('end-user-123');
    expect(endUser?.deviceId).toBe('existing-device-id');
    expect(conversation?.id).toBe('conv-789');
  });

  it('should save session to localStorage after init', async () => {
    // Arrange
    vi.mocked(localStorage.getItem).mockReturnValue(null);

    widget = new FeedbackWidget({
      apiKey: 'test-key',
      projectId: 'project-123',
      apiUrl: 'http://localhost:3000',
    });

    await widget.init();

    // Verify localStorage.setItem was called
    expect(localStorage.setItem).toHaveBeenCalled();
    const setItemCall = mockFetch.mock.calls[0];
    // The last call to setItem should contain session data
    const sessionKey = (localStorage.setItem as ReturnType<typeof vi.fn>).mock.calls.find(
      (call: unknown[]) => call[0] === 'feedback_sdk_session'
    );
    expect(sessionKey).toBeDefined();
    const sessionData = JSON.parse(sessionKey[1]);
    expect(sessionData).toHaveProperty('projectId', 'project-123');
    expect(sessionData).toHaveProperty('deviceId');
    expect(sessionData).toHaveProperty('endUserId', 'end-user-123');
    expect(sessionData).toHaveProperty('conversationId', 'conv-789');
  });

  it('should call /api/v1/sdk/init with device_id', async () => {
    // Arrange
    vi.mocked(localStorage.getItem).mockReturnValue(null);

    widget = new FeedbackWidget({
      apiKey: 'test-key',
      projectId: 'project-123',
      apiUrl: 'http://localhost:3000',
    });

    await widget.init();

    // Verify fetch was called with correct path
    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:3000/api/v1/sdk/init',
      expect.objectContaining({
        method: 'POST',
      })
    );
  });

  it('should not re-init if already initialized', async () => {
    // Arrange
    vi.mocked(localStorage.getItem).mockReturnValue(null);

    widget = new FeedbackWidget({
      apiKey: 'test-key',
      projectId: 'project-123',
      apiUrl: 'http://localhost:3000',
    });

    await widget.init();
    await widget.init();

    // Should only call fetch once
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });
});
