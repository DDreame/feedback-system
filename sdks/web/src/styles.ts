/**
 * Feedback Widget Styles - Encapsulated in Shadow DOM
 */

export const widgetStyles = `
.feedback-widget {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: #333;
  position: fixed;
  z-index: 999999;
  bottom: 20px;
  right: 20px;
}

.feedback-widget * {
  box-sizing: border-box;
}

/* Floating Button */
.feedback-floating-button {
  width: 56px;
  height: 56px;
  border-radius: 28px;
  background-color: #4F46E5;
  color: white;
  border: none;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s, box-shadow 0.2s;
}

.feedback-floating-button:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 16px rgba(79, 70, 229, 0.5);
}

.feedback-floating-button:active {
  transform: scale(0.95);
}

.feedback-floating-button svg {
  width: 24px;
  height: 24px;
}

/* Chat Window */
.feedback-chat-window {
  position: absolute;
  bottom: 70px;
  right: 0;
  width: 360px;
  height: 480px;
  background-color: white;
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  display: none;
  flex-direction: column;
  overflow: hidden;
}

.feedback-chat-window.open {
  display: flex;
}

/* Header */
.feedback-header {
  background-color: #4F46E5;
  color: white;
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.feedback-header-title {
  font-weight: 600;
  font-size: 16px;
}

.feedback-header-status {
  font-size: 12px;
  opacity: 0.9;
  display: flex;
  align-items: center;
  gap: 4px;
}

.feedback-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #10B981;
}

.feedback-status-dot.disconnected {
  background-color: #EF4444;
}

/* Messages Area */
.feedback-messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background-color: #F9FAFB;
}

.feedback-message {
  margin-bottom: 12px;
  max-width: 80%;
}

.feedback-message.user {
  margin-left: auto;
}

.feedback-message-content {
  padding: 10px 14px;
  border-radius: 16px;
  word-wrap: break-word;
}

.feedback-message.user .feedback-message-content {
  background-color: #4F46E5;
  color: white;
  border-bottom-right-radius: 4px;
}

.feedback-message.developer .feedback-message-content,
.feedback-message.ai_agent .feedback-message-content {
  background-color: white;
  border: 1px solid #E5E7EB;
  border-bottom-left-radius: 4px;
}

.feedback-message-time {
  font-size: 11px;
  color: #9CA3AF;
  margin-top: 4px;
  text-align: right;
}

/* Input Area */
.feedback-input-area {
  padding: 12px;
  border-top: 1px solid #E5E7EB;
  display: flex;
  gap: 8px;
}

.feedback-input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid #E5E7EB;
  border-radius: 20px;
  outline: none;
  font-size: 14px;
  resize: none;
  max-height: 100px;
  font-family: inherit;
}

.feedback-input:focus {
  border-color: #4F46E5;
}

.feedback-send-button {
  width: 36px;
  height: 36px;
  border-radius: 18px;
  background-color: #4F46E5;
  color: white;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s;
}

.feedback-send-button:hover {
  background-color: #4338CA;
}

.feedback-send-button:disabled {
  background-color: #9CA3AF;
  cursor: not-allowed;
}

.feedback-send-button svg {
  width: 18px;
  height: 18px;
}

/* Scrollbar styling */
.feedback-messages::-webkit-scrollbar {
  width: 6px;
}

.feedback-messages::-webkit-scrollbar-track {
  background: transparent;
}

.feedback-messages::-webkit-scrollbar-thumb {
  background-color: #D1D5DB;
  border-radius: 3px;
}

/* Empty state */
.feedback-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #9CA3AF;
  text-align: center;
  padding: 20px;
}

.feedback-empty-state-icon {
  width: 48px;
  height: 48px;
  margin-bottom: 12px;
  opacity: 0.5;
}
`;
