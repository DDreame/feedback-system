import { describe, it, expect, beforeEach, vi } from 'vitest';
import { FeedbackWidget, FeedbackWidgetConfig } from '../src/index';

describe('FeedbackWidget', () => {
  let widget: FeedbackWidget;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('constructor - config validation', () => {
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

    it('should throw error when apiKey is empty string', () => {
      expect(() => {
        new FeedbackWidget({ apiKey: '', projectId: 'test-project' } as FeedbackWidgetConfig);
      }).toThrow('apiKey is required');
    });

    it('should throw error when projectId is empty string', () => {
      expect(() => {
        new FeedbackWidget({ apiKey: 'test-key', projectId: '' } as FeedbackWidgetConfig);
      }).toThrow('projectId is required');
    });

    it('should throw error when apiUrl is invalid URL format', () => {
      expect(() => {
        new FeedbackWidget({
          apiKey: 'test-key',
          projectId: 'test-project',
          apiUrl: 'not-a-valid-url',
        });
      }).toThrow('apiUrl must be a valid URL');
    });

    it('should throw error when wsUrl is invalid URL format', () => {
      expect(() => {
        new FeedbackWidget({
          apiKey: 'test-key',
          projectId: 'test-project',
          wsUrl: 'not-a-valid-url',
        });
      }).toThrow('wsUrl must be a valid URL');
    });

    it('should accept valid apiUrl', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        apiUrl: 'https://custom.api.com',
      });
      expect(widget).toBeDefined();
    });

    it('should accept valid wsUrl', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        wsUrl: 'wss://custom.ws.com',
      });
      expect(widget).toBeDefined();
    });

    it('should use default apiTimeout of 30000ms', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget).toBeDefined();
    });

    it('should accept custom apiTimeout', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        apiTimeout: 5000,
      });
      expect(widget).toBeDefined();
    });

    it('should default debug to false', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
      });
      expect(widget).toBeDefined();
    });

    it('should accept custom debug setting', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        debug: true,
      });
      expect(widget).toBeDefined();
    });

    it('should accept custom container as string selector', () => {
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        container: '#my-widget',
      });
      expect(widget).toBeDefined();
    });

    it('should accept custom container as HTMLElement', () => {
      const container = document.createElement('div');
      const widget = new FeedbackWidget({
        apiKey: 'test-key',
        projectId: 'test-project',
        container,
      });
      expect(widget).toBeDefined();
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
