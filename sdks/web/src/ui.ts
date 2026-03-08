/**
 * Feedback Widget UI - Shadow DOM rendering
 */

import { widgetStyles } from './styles';
import { Message } from './index';

export interface ThemeConfig {
  primaryColor?: string;
  fontFamily?: string;
  position?: 'bottom-right' | 'bottom-left';
  buttonIcon?: string;
}

export class WidgetUI {
  private container: HTMLElement;
  private shadowRoot: ShadowRoot;
  private widgetElement: HTMLElement;
  private floatingButton: HTMLButtonElement;
  private chatWindow: HTMLElement;
  private messagesContainer: HTMLElement;
  private inputElement: HTMLTextAreaElement;
  private sendButton: HTMLButtonElement;
  private isOpen = false;
  private onSendMessage?: (content: string) => void;
  private isConnected = false;

  constructor(container: HTMLElement) {
    this.container = container;

    // Create shadow root for style isolation (or reuse existing one)
    if (this.container.shadowRoot) {
      this.shadowRoot = this.container.shadowRoot;
    } else {
      this.shadowRoot = this.container.attachShadow({ mode: 'open' });
    }

    // Create widget root element (or find existing one)
    const existingWidget = this.shadowRoot.querySelector('.feedback-widget');
    if (existingWidget) {
      this.widgetElement = existingWidget as HTMLElement;
    } else {
      this.widgetElement = document.createElement('div');
      this.widgetElement.className = 'feedback-widget';

      // Create styles
      const styleElement = document.createElement('style');
      styleElement.textContent = widgetStyles;
      this.shadowRoot.appendChild(styleElement);
      this.shadowRoot.appendChild(this.widgetElement);
    }

    // Create floating button (or find existing one)
    const existingButton = this.widgetElement.querySelector('.feedback-floating-button');
    if (existingButton) {
      this.floatingButton = existingButton as HTMLButtonElement;
    } else {
      this.floatingButton = this.createFloatingButton();
      this.widgetElement.appendChild(this.floatingButton);
    }

    // Create chat window (or find existing one)
    const existingWindow = this.widgetElement.querySelector('.feedback-chat-window');
    if (existingWindow) {
      this.chatWindow = existingWindow as HTMLElement;
      // Re-initialize messages container reference
      this.messagesContainer = this.chatWindow.querySelector('.feedback-messages') as HTMLElement;
      this.inputElement = this.chatWindow.querySelector('.feedback-input') as HTMLTextAreaElement;
      this.sendButton = this.chatWindow.querySelector('.feedback-send-button') as HTMLButtonElement;
    } else {
      this.chatWindow = this.createChatWindow();
      this.widgetElement.appendChild(this.chatWindow);
    }

    // Bind click handler for toggle
    this.floatingButton.addEventListener('click', () => this.toggle());
  }

  /**
   * Set the message send handler
   */
  setOnSendMessage(handler: (content: string) => void): void {
    this.onSendMessage = handler;
  }

  /**
   * Set connection status
   */
  setConnected(connected: boolean): void {
    this.isConnected = connected;
    const statusDot = this.chatWindow.querySelector('.feedback-status-dot') as HTMLElement;
    const statusText = this.chatWindow.querySelector('.feedback-header-status') as HTMLElement;

    if (statusDot) {
      statusDot.classList.toggle('disconnected', !connected);
    }
    if (statusText) {
      statusText.textContent = connected ? 'Connected' : 'Disconnected';
    }
  }

  /**
   * Toggle chat window visibility
   */
  toggle(): void {
    this.isOpen = !this.isOpen;
    this.chatWindow.classList.toggle('open', this.isOpen);

    // Update button icon
    if (this.isOpen) {
      this.floatingButton.innerHTML = this.getCloseIcon();
    } else {
      this.floatingButton.innerHTML = this.getChatIcon();
    }

    // Focus input when opened
    if (this.isOpen) {
      setTimeout(() => this.inputElement?.focus(), 100);
    }
  }

  /**
   * Add a message to the chat
   */
  addMessage(message: Message): void {
    // Remove empty state if present
    const emptyState = this.messagesContainer.querySelector('.feedback-empty-state');
    if (emptyState) {
      emptyState.remove();
    }

    const messageElement = document.createElement('div');
    messageElement.className = `feedback-message ${message.senderType === 'end_user' ? 'user' : message.senderType}`;

    const contentElement = document.createElement('div');
    contentElement.className = 'feedback-message-content';
    contentElement.textContent = message.content;

    const timeElement = document.createElement('div');
    timeElement.className = 'feedback-message-time';
    timeElement.textContent = this.formatTime(message.createdAt);

    messageElement.appendChild(contentElement);
    messageElement.appendChild(timeElement);

    this.messagesContainer.appendChild(messageElement);

    // Scroll to bottom
    this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
  }

  /**
   * Clear all messages
   */
  clearMessages(): void {
    this.messagesContainer.innerHTML = '';
  }

  /**
   * Get the widget element for positioning
   */
  getWidgetElement(): HTMLElement {
    return this.widgetElement;
  }

  /**
   * Destroy the UI
   */
  destroy(): void {
    this.floatingButton.removeEventListener('click', () => this.toggle());
    this.sendButton.removeEventListener('click', () => this.handleSend());
    this.inputElement.removeEventListener('keydown', (e) => this.handleKeyDown(e));
  }

  private createFloatingButton(): HTMLButtonElement {
    const button = document.createElement('button');
    button.className = 'feedback-floating-button';
    button.setAttribute('aria-label', 'Open chat');
    button.innerHTML = this.getChatIcon();
    return button;
  }

  private createChatWindow(): HTMLElement {
    const window = document.createElement('div');
    window.className = 'feedback-chat-window';

    // Header
    const header = document.createElement('div');
    header.className = 'feedback-header';
    header.innerHTML = `
      <div>
        <div class="feedback-header-title">Support Chat</div>
        <div class="feedback-header-status">
          <span class="feedback-status-dot"></span>
          <span>Connected</span>
        </div>
      </div>
    `;
    window.appendChild(header);

    // Messages area
    this.messagesContainer = document.createElement('div');
    this.messagesContainer.className = 'feedback-messages';
    this.messagesContainer.innerHTML = `
      <div class="feedback-empty-state">
        <svg class="feedback-empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
        </svg>
        <div>Send us a message and we'll get back to you!</div>
      </div>
    `;
    window.appendChild(this.messagesContainer);

    // Input area
    const inputArea = document.createElement('div');
    inputArea.className = 'feedback-input-area';

    this.inputElement = document.createElement('textarea');
    this.inputElement.className = 'feedback-input';
    this.inputElement.placeholder = 'Type a message...';
    this.inputElement.rows = 1;
    this.inputElement.addEventListener('keydown', (e) => this.handleKeyDown(e));

    this.sendButton = document.createElement('button');
    this.sendButton.className = 'feedback-send-button';
    this.sendButton.setAttribute('aria-label', 'Send message');
    this.sendButton.innerHTML = this.getSendIcon();
    this.sendButton.addEventListener('click', () => this.handleSend());

    inputArea.appendChild(this.inputElement);
    inputArea.appendChild(this.sendButton);
    window.appendChild(inputArea);

    return window;
  }

  private handleKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      this.handleSend();
    }
  }

  private handleSend(): void {
    const content = this.inputElement.value.trim();
    if (content && this.onSendMessage) {
      this.onSendMessage(content);
      this.inputElement.value = '';
      this.inputElement.style.height = 'auto';
    }
  }

  private formatTime(isoString: string): string {
    const date = new Date(isoString);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  private getChatIcon(): string {
    return `
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
      </svg>
    `;
  }

  private getCloseIcon(): string {
    return `
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    `;
  }

  private getSendIcon(): string {
    return `
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="22" y1="2" x2="11" y2="13"></line>
        <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
      </svg>
    `;
  }
}
