import type { ReactNode } from 'react';
import { AppHeader } from './AppHeader';

interface AppLayoutProps {
  children: ReactNode;
}

/**
 * Main application layout component.
 * 
 * Responsibilities:
 * - Define overall page structure
 * - Coordinate header, breadcrumb, content, and footer
 * 
 * Benefits:
 * - Single source of truth for app layout
 * - Easy to modify layout structure
 * - Reusable across all pages
 */
export function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="flex-1 relative overflow-hidden bg-background dark:bg-background/80">
      <AppHeader />
      <main
        className="absolute inset-0 overflow-y-auto overflow-x-hidden"
        style={{ paddingTop: 'var(--chrome-header-height)' }}
      >
        {children}
      </main>
    </div>
  );
}
