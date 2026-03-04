import { motion } from 'motion/react';
import { X } from 'lucide-react';

interface Tab {
  id: string;
  title: string;
  url: string;
  isActive: boolean;
}

interface TabBarProps {
  tabs: Tab[];
  onTabClick: (id: string) => void;
  onTabClose: (id: string) => void;
  onNewTab: () => void;
}

export function TabBar({ tabs, onTabClick, onTabClose, onNewTab }: TabBarProps) {
  return (
    <div 
      className="flex items-center gap-0.5 overflow-x-auto px-2 py-1"
      style={{
        fontFamily: 'var(--font-tension-sans)',
        background: 'var(--temp-bg-secondary)',
      }}
    >
      {tabs.map((tab) => (
        <motion.div
          key={tab.id}
          layout
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.9 }}
          transition={{
            type: "spring",
            stiffness: 300,
            damping: 25,
          }}
          className="relative group"
        >
          <div
            onClick={() => onTabClick(tab.id)}
            className="relative flex items-center gap-2 px-4 py-2 min-w-[120px] max-w-[240px] transition-colors cursor-pointer"
            style={{
              background: tab.isActive ? 'var(--temp-bg-primary)' : 'transparent',
              color: 'var(--temp-text-primary)',
              clipPath: 'polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px)',
            }}
          >
            <span className="truncate">{tab.title}</span>
            <motion.button
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              onClick={(e) => {
                e.stopPropagation();
                onTabClose(tab.id);
              }}
              className="opacity-0 group-hover:opacity-100 p-1 hover:bg-black/10 rounded-sm transition-opacity"
            >
              <X size={12} />
            </motion.button>
          </div>
        </motion.div>
      ))}
      
      <motion.button
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
        onClick={onNewTab}
        className="px-3 py-2 hover:bg-black/5 transition-colors"
        style={{
          color: 'var(--temp-text-secondary)',
          clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)',
        }}
      >
        <span className="text-lg leading-none">+</span>
      </motion.button>
    </div>
  );
}