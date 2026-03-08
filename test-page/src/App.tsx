import { FeedbackWidget } from '@feedback-system/web-sdk';
import { useEffect, useState } from 'react';

function App() {
  const [message, setMessage] = useState<string>('');
  const [status, setStatus] = useState<string>('Initializing...');
  const [connected, setConnected] = useState<boolean>(false);

  useEffect(() => {
    // Initialize the feedback widget
    const widget = new FeedbackWidget({
      apiKey: import.meta.env.VITE_API_KEY || 'test-api-key',
      projectId: import.meta.env.VITE_PROJECT_ID || 'test-project-id',
      apiUrl: import.meta.env.VITE_API_URL || 'http://localhost:3000',
      debug: true,
      onMessage: (msg) => {
        console.log('New message:', msg);
        setMessage(msg.content);
      },
      onConnectionChange: (isConnected) => {
        console.log('Connection changed:', isConnected);
        setConnected(isConnected);
        setStatus(isConnected ? 'Connected' : 'Disconnected');
      },
    });

    widget.init().catch((err) => {
      console.error('Init error:', err);
      setStatus(`Error: ${err.message}`);
    });

    return () => {
      widget.destroy();
    };
  }, []);

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui' }}>
      <h1>Feedback System Test Page</h1>
      <div style={{ marginBottom: '10px' }}>
        <strong>Status:</strong> {status}
      </div>
      <div style={{ marginBottom: '10px' }}>
        <strong>Connected:</strong> {connected ? 'Yes' : 'No'}
      </div>
      {message && (
        <div style={{ padding: '10px', background: '#f0f0f0', borderRadius: '4px' }}>
          <strong>Latest Message:</strong> {message}
        </div>
      )}
    </div>
  );
}

export default App;
