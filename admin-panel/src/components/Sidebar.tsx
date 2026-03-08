import { NavLink } from 'react-router-dom';

function Sidebar() {
  const navItems = [
    { to: '/', label: 'Dashboard', icon: '📊' },
    { to: '/projects', label: 'Projects', icon: '📁' },
    { to: '/inbox', label: 'Inbox', icon: '📥' },
    { to: '/settings', label: 'Settings', icon: '⚙️' },
  ];

  return (
    <nav style={{
      width: '240px',
      backgroundColor: '#1a1a2e',
      color: 'white',
      padding: '24px 16px',
      display: 'flex',
      flexDirection: 'column',
    }}>
      <h1 style={{ fontSize: '20px', marginBottom: '32px', padding: '0 12px' }}>
        Feedback System
      </h1>
      <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
        {navItems.map((item) => (
          <li key={item.to} style={{ marginBottom: '8px' }}>
            <NavLink
              to={item.to}
              style={({ isActive }) => ({
                display: 'block',
                padding: '12px',
                borderRadius: '8px',
                color: 'white',
                textDecoration: 'none',
                backgroundColor: isActive ? '#16213e' : 'transparent',
                transition: 'background-color 0.2s',
              })}
            >
              <span style={{ marginRight: '12px' }}>{item.icon}</span>
              {item.label}
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}

export default Sidebar;
