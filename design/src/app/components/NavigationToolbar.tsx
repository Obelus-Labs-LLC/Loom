import { motion } from 'motion/react';
import { ChevronLeft, ChevronRight, RotateCw, Home, Menu, Settings } from 'lucide-react';

interface NavigationToolbarProps {
  onBack: () => void;
  onForward: () => void;
  onRefresh: () => void;
  onHome: () => void;
  onMenu: () => void;
  onSettings: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
}

export function NavigationToolbar({
  onBack,
  onForward,
  onRefresh,
  onHome,
  onMenu,
  onSettings,
  canGoBack,
  canGoForward,
}: NavigationToolbarProps) {
  const NavButton = ({ 
    onClick, 
    disabled, 
    children, 
    label 
  }: { 
    onClick: () => void; 
    disabled?: boolean; 
    children: React.ReactNode; 
    label: string 
  }) => (
    <motion.button
      whileHover={!disabled ? { scale: 1.05 } : {}}
      whileTap={!disabled ? { scale: 0.95 } : {}}
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
      className="p-2 hover:bg-black/5 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
      style={{
        color: 'var(--temp-text-primary)',
        clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)',
      }}
    >
      {children}
    </motion.button>
  );

  return (
    <div 
      className="flex items-center gap-1 px-2"
      style={{ fontFamily: 'var(--font-tension-sans)' }}
    >
      <NavButton onClick={onBack} disabled={!canGoBack} label="Go back">
        <ChevronLeft size={20} />
      </NavButton>
      <NavButton onClick={onForward} disabled={!canGoForward} label="Go forward">
        <ChevronRight size={20} />
      </NavButton>
      <NavButton onClick={onRefresh} label="Refresh">
        <RotateCw size={18} />
      </NavButton>
      <NavButton onClick={onHome} label="Home">
        <Home size={18} />
      </NavButton>
      
      <div className="flex-1" />
      
      <NavButton onClick={onMenu} label="Menu">
        <Menu size={18} />
      </NavButton>
      <NavButton onClick={onSettings} label="Settings">
        <Settings size={18} />
      </NavButton>
    </div>
  );
}
