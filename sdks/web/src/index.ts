/**
 * Feedback System Web SDK
 * Provides a widget for end users to communicate with developers.
 */

import { HttpClient } from './http';

export interface FeedbackWidgetConfig {
  /** The API key from the project settings (required) */
  apiKey: string;
  /** The project ID (required) */
  projectId: string;
  /** Optional: Custom API URL (defaults to current origin) */
  apiUrl?: string;
  /** Optional: Custom WebSocket URL (defaults to current origin) */
  wsUrl?: string;
  /** Optional: Container element to mount the widget */
  container?: HTMLElement | string;
  /** Optional: API timeout in ms (defaults to 30000ms) */
  apiTimeout?: number;
  /** Optional: Enable debug logging (defaults to false) */
  debug?: boolean;
  /** Optional: Custom message handler */
  onMessage?: MessageHandler;
  /** Optional: Custom connection handler */
  onConnectionChange?: ConnectionHandler;
}

export interface Message {
  id: string;
  conversationId: string;
  senderType: 'developer' | 'end_user' | 'ai_agent';
  senderId?: string;
  messageType: 'text' | 'system';
  content: string;
  createdAt: string;
}

export interface EndUser {
  id: string;
  projectId: string;
  deviceId: string;
  name?: string;
}

export interface Conversation {
  id: string;
  projectId: string;
  endUserId: string;
  status: 'open' | 'closed';
}

export interface InitResponse {
  endUser: EndUser;
  conversation: Conversation;
}

export type MessageHandler = (message: Message) => void;
export type ConnectionHandler = (connected: boolean) => void;

/**
 * FeedbackWidget - Main class for the web SDK
 */
export class FeedbackWidget {
  private config: FeedbackWidgetConfig & { apiUrl: string; wsUrl: string; apiTimeout: number; debug: boolean };
  private httpClient: HttpClient;
  private endUser?: EndUser;
  private conversation?: Conversation;
  private messageHandlers: Set<MessageHandler> = new Set();
  private connectionHandlers: Set<ConnectionHandler> = new Set();
  private ws?: WebSocket;
  private initialized = false;

  constructor(config: FeedbackWidgetConfig) {
    // Validate required config
    if (!config.apiKey || config.apiKey.trim() === '') {
      throw new Error('apiKey is required');
    }
    if (!config.projectId || config.projectId.trim() === '') {
      throw new Error('projectId is required');
    }

    // Validate apiUrl if provided
    if (config.apiUrl !== undefined) {
      try {
        new URL(config.apiUrl);
      } catch {
        throw new Error('apiUrl must be a valid URL');
      }
    }

    // Validate wsUrl if provided
    if (config.wsUrl !== undefined) {
      try {
        new URL(config.wsUrl);
      } catch {
        throw new Error('wsUrl must be a valid URL');
      }
    }

    // Set defaults
    const defaultApiUrl = typeof self !== 'undefined' ? self.location.origin : 'http://localhost:3000';
    const defaultWsUrl = typeof self !== 'undefined' ? self.location.origin : 'http://localhost:3000';

    this.config = {
      apiKey: config.apiKey,
      projectId: config.projectId,
      apiUrl: config.apiUrl || defaultApiUrl,
      wsUrl: config.wsUrl || defaultWsUrl,
      container: config.container || (typeof document !== 'undefined' ? document.body : undefined),
      apiTimeout: config.apiTimeout ?? 30000,
      debug: config.debug ?? false,
      onMessage: config.onMessage,
      onConnectionChange: config.onConnectionChange,
    };

    // Initialize HTTP client
    this.httpClient = new HttpClient({
      apiKey: this.config.apiKey,
      baseUrl: this.config.apiUrl,
      timeout: this.config.apiTimeout,
      debug: this.config.debug,
    });
  }

  /**
   * Initialize the SDK - must be called before using other methods
   */
  async init(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Call the SDK init endpoint
      const data: InitResponse = await this.httpClient.post('/api/v1/sdk/init', {
        project_id: this.config.projectId,
      });

      this.endUser = {
        id: data.endUser.id,
        projectId: data.endUser.projectId,
        deviceId: data.endUser.deviceId,
        name: data.endUser.name,
      };

      this.conversation = {
        id: data.conversation.id,
        projectId: data.conversation.projectId,
        endUserId: data.conversation.endUserId,
        status: data.conversation.status,
      };

      this.initialized = true;

      // Connect to WebSocket
      this.connectWebSocket();
    } catch (error) {
      throw new Error(`Failed to initialize: ${error}`);
    }
  }

  /**
   * Connect to WebSocket for real-time messaging
   */
  private connectWebSocket(): void {
    if (!this.conversation) {
      return;
    }

    const wsUrl = `${this.config.wsUrl}/api/v1/sdk/ws?conversation_id=${this.conversation.id}&end_user_id=${this.endUser?.id}`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      this.connectionHandlers.forEach(handler => handler(true));
    };

    this.ws.onclose = () => {
      this.connectionHandlers.forEach(handler => handler(false));
      // TODO: Implement reconnection logic
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'new_message' && data.message) {
          const message: Message = {
            id: data.message.id,
            conversationId: data.message.conversation_id,
            senderType: data.message.sender_type,
            senderId: data.message.sender_id,
            messageType: data.message.message_type,
            content: data.message.content,
            createdAt: data.message.created_at,
          };
          this.messageHandlers.forEach(handler => handler(message));
        }
      } catch (e) {
        console.error('Failed to parse message:', e);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  /**
   * Send a message
   */
  async sendMessage(content: string): Promise<Message> {
    if (!this.initialized || !this.conversation) {
      throw new Error('SDK not initialized');
    }

    const data = await this.httpClient.post<{ message: Message }>('/api/v1/sdk/messages', {
      conversation_id: this.conversation.id,
      message_type: 'text',
      content,
    });

    return data.message;
  }

  /**
   * Register a handler for incoming messages
   */
  onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  /**
   * Register a handler for connection status changes
   */
  onConnectionChange(handler: ConnectionHandler): () => void {
    this.connectionHandlers.add(handler);
    return () => this.connectionHandlers.delete(handler);
  }

  /**
   * Get the current end user
   */
  getEndUser(): EndUser | undefined {
    return this.endUser;
  }

  /**
   * Get the current conversation
   */
  getConversation(): Conversation | undefined {
    return this.conversation;
  }

  /**
   * Check if connected via WebSocket
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Disconnect and cleanup
   */
  destroy(): void {
    this.ws?.close();
    this.ws = undefined;
    this.messageHandlers.clear();
    this.connectionHandlers.clear();
    this.initialized = false;
  }
}

export default FeedbackWidget;
