import { describe, it, expect, beforeEach, vi } from 'vitest';
import { FeedbackWidget, FeedbackWidgetConfig } from '../src/index';

describe('FeedbackWidget', () => {
  let widget: FeedbackWidget;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('should throw error when apiKey is missing', () => {
      expect(() => {
        new FeedbackWidget({ projectId: 'test-project' } as FeedbackWidgetConfig);
      }).toThrow('apiKey is required');
    });

    it('should throw error when projectId is missing', () => {
      expect(() => {
        new FeedbackWidget({ apiKey: 'test-key' } as FeedbackWidgetConfig);
      }).toThrow('projectId is required');
    });

    it('should create widget with valid config', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget).toBeDefined();
    });

    it('should use default values for optional config', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget).toBeDefined();
    });

    it('should accept custom apiUrl', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        apiUrl: 'https://custom.api',
      });
      expect(widget).toBeDefined();
    });

    it('should accept custom wsUrl', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        wsUrl: 'wss://custom.ws',
      });
      expect(widget).toBeDefined();
    });
  });

  describe('getEndUser', () => {
    it('should return undefined before init', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget.getEndUser()).toBeUndefined();
    });
  });

  describe('getConversation', () => {
    it('should return undefined before init', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget.getConversation()).toBeUndefined();
    });
  });

  describe('isConnected', () => {
    it('should return false before init', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget.isConnected()).toBe(false);
    });
  });

  describe('onMessage', () => {
    it('should register and return unsubscribe function', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      const handler = vi.fn();
      const unsubscribe = widget.onMessage(handler);

      expect(typeof unsubscribe).toBe('function');
      unsubscribe();
    });
  });

  describe('onConnectionChange', () => {
    it('should register and return unsubscribe function', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      const handler = vi.fn();
      const unsubscribe = widget.onConnectionChange(handler);

      expect(typeof unsubscribe).toBe('function');
      unsubscribe();
    });
  });

  describe('destroy', () => {
    it('should cleanup resources', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      widget.destroy();
      expect(widget.isConnected()).toBe(false);
    });
  });
});
