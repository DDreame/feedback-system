import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import App from '../App';

// Mock the FeedbackWidget
vi.mock('@feedback-system/web-sdk', () => ({
  FeedbackWidget: vi.fn().mockImplementation(() => ({
    init: vi.fn().mockResolvedValue(undefined),
    destroy: vi.fn(),
    sendMessage: vi.fn().mockResolvedValue({
      id: 'test-message-id',
      conversationId: 'test-conversation-id',
      senderType: 'end_user',
      messageType: 'text',
      content: 'Test message',
      createdAt: new Date().toISOString(),
    }),
    onMessage: vi.fn(),
    onConnectionChange: vi.fn(),
    getEndUser: vi.fn().mockReturnValue({
      id: 'test-user-id',
      projectId: 'test-project-id',
      deviceId: 'test-device-id',
    }),
    getConversation: vi.fn().mockReturnValue({
      id: 'test-conversation-id',
      projectId: 'test-project-id',
      endUserId: 'test-user-id',
      status: 'open',
    }),
    isConnected: vi.fn().mockReturnValue(false),
  })),
}));

// Mock import.meta.env
vi.mock('import.meta.env', () => ({
  env: {
    VITE_API_KEY: '',
    VITE_PROJECT_ID: '',
    VITE_API_URL: 'http://localhost:3000',
  },
}));

describe('App', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    vi.stubGlobal('localStorage', {
      getItem: vi.fn().mockReturnValue(null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    });

    // Mock crypto.randomUUID
    vi.stubGlobal('crypto', {
      randomUUID: vi.fn().mockReturnValue('test-uuid-1234'),
    });
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  const renderWithRouter = (component: React.ReactElement) => {
    return render(<BrowserRouter>{component}</BrowserRouter>);
  };

  it('renders the SDK simulator page title', () => {
    renderWithRouter(<App />);
    expect(screen.getByText('Feedback System SDK Simulator')).toBeInTheDocument();
  });

  it('renders configuration inputs', () => {
    renderWithRouter(<App />);

    // Check for configuration inputs
    expect(screen.getByLabelText(/API Key/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Project ID/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Device ID/i)).toBeInTheDocument();
  });

  it('renders connect button', () => {
    renderWithRouter(<App />);
    expect(screen.getByRole('button', { name: /connect/i })).toBeInTheDocument();
  });

  it('shows initial disconnected status', () => {
    renderWithRouter(<App />);
    expect(screen.getByText(/not connected/i)).toBeInTheDocument();
  });
});
