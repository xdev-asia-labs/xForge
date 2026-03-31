import { NavLink, Outlet } from 'react-router-dom';

interface LayoutProps {
  onLogout: () => void;
}

const navItems = [
  { path: '/', label: 'Dashboard', icon: '◆' },
  { path: '/servers', label: 'Servers', icon: '⬡' },
  { path: '/recipes', label: 'Recipes', icon: '◈' },
  { path: '/jobs', label: 'Jobs', icon: '▶' },
  { path: '/marketplace', label: 'Marketplace', icon: '⬙' },
];

export default function Layout({ onLogout }: LayoutProps) {
  return (
    <div className="flex h-screen bg-gray-950">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
        <div className="p-6 border-b border-gray-800">
          <h1 className="text-2xl font-bold text-forge-400">
            ⚒ xForge
          </h1>
          <p className="text-xs text-gray-500 mt-1">Infrastructure Management</p>
        </div>

        <nav className="flex-1 p-4 space-y-1">
          {navItems.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === '/'}
              className={({ isActive }) =>
                `flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                  isActive
                    ? 'bg-forge-600/20 text-forge-400'
                    : 'text-gray-400 hover:text-white hover:bg-gray-800'
                }`
              }
            >
              <span className="text-lg">{item.icon}</span>
              {item.label}
            </NavLink>
          ))}
        </nav>

        <div className="p-4 border-t border-gray-800">
          <button
            onClick={onLogout}
            className="w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors"
          >
            <span className="text-lg">⏻</span>
            Logout
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <div className="p-8">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
