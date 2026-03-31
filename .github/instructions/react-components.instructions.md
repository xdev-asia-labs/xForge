---
description: "Use when creating React components, pages, or modifying the frontend UI. Covers TailwindCSS dark theme classes and component structure."
applyTo: "web/src/**/*.tsx"
---
# React Component Patterns

## Component Structure
- Export default function components (not arrow functions)
- Explicit generic types on `useState<T>()`
- Load data in `useEffect(() => { loadFn(); }, [])`

## Error Handling
```tsx
try {
  await apiCall();
} catch (err) {
  setError(err instanceof Error ? err.message : 'Fallback message');
}
```

## TailwindCSS Classes (dark theme only)

Page container: no wrapper needed, `<Outlet>` inside Layout provides `p-8`

Cards:
```
bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors
```

Form inputs:
```
w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm
focus:outline-none focus:ring-2 focus:ring-forge-500
```

Primary buttons:
```
px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors
```

Error banners:
```
bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg
```

Status badges use transparency: `bg-green-600/20 text-green-400` for success, `bg-red-600/20 text-red-400` for failed, `bg-yellow-600/20 text-yellow-400` for running.

Labels/tags: `px-2 py-0.5 bg-forge-600/20 text-forge-400 text-xs rounded-md`
