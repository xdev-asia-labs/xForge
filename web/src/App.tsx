import { useEffect, useState } from 'react';
import { Navigate, Route, Routes } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Jobs from './pages/Jobs';
import KeyStore from './pages/KeyStore';
import Login from './pages/Login';
import Marketplace from './pages/Marketplace';
import Recipes from './pages/Recipes';
import Schedules from './pages/Schedules';
import Servers from './pages/Servers';
import SecurityAudit from './pages/SecurityAudit';
import ServerTerminal from './pages/ServerTerminal';
import Settings from './pages/Settings';
import Users from './pages/Users';

function App() {
  const [token, setToken] = useState<string | null>(
    localStorage.getItem('xforge_token')
  );

  useEffect(() => {
    if (token) {
      localStorage.setItem('xforge_token', token);
    } else {
      localStorage.removeItem('xforge_token');
    }
  }, [token]);

  const handleLogout = () => {
    setToken(null);
  };

  if (!token) {
    return <Login onLogin={setToken} />;
  }

  return (
    <Routes>
      <Route path="/" element={<Layout onLogout={handleLogout} />}>
        <Route index element={<Dashboard />} />
        <Route path="servers" element={<Servers />} />
        <Route path="recipes" element={<Recipes />} />
        <Route path="jobs" element={<Jobs />} />
        <Route path="schedules" element={<Schedules />} />
        <Route path="keys" element={<KeyStore />} />
        <Route path="marketplace" element={<Marketplace />} />
        <Route path="users" element={<Users />} />
        <Route path="settings" element={<Settings />} />
        <Route path="terminal/:id" element={<ServerTerminal />} />
        <Route path="servers/:id/audit" element={<SecurityAudit />} />
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default App;
