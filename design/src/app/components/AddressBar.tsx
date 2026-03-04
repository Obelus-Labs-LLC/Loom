import { motion, AnimatePresence } from 'motion/react';
import { Search, Lock, RefreshCw } from 'lucide-react';
import { useState, KeyboardEvent } from 'react';

interface AddressBarProps {
  value: string;
  onChange: (value: string) => void;
  onNavigate: (url: string) => void;
  isSecure?: boolean;
  isLoading?: boolean;
  mode: 'traditional' | 'ai';
}

export function AddressBar({ 
  value, 
  onChange, 
  onNavigate, 
  isSecure = true,
  isLoading = false,
  mode 
}: AddressBarProps) {
  const [isFocused, setIsFocused] = useState(false);

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      onNavigate(value);
    }
  };

  const placeholder = mode === 'ai' 
    ? 'What do you want to find?' 
    : 'Search or enter address';

  return (
    <motion.div 
      layout
      className="relative flex items-center gap-2 px-4 py-2 w-full"
      style={{
        background: isFocused ? 'var(--temp-bg-primary)' : 'var(--temp-bg-secondary)',
        fontFamily: mode === 'ai' ? 'var(--font-hand)' : 'var(--font-tension-sans)',
        clipPath: 'polygon(12px 0, calc(100% - 12px) 0, 100% 12px, 100% calc(100% - 12px), calc(100% - 12px) 100%, 12px 100%, 0 calc(100% - 12px), 0 12px)',
      }}
      transition={{
        type: "spring",
        stiffness: 300,
        damping: 30,
      }}
    >
      {/* Security indicator */}
      <AnimatePresence>
        {mode === 'traditional' && (
          <motion.div
            initial={{ opacity: 0, scale: 0 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0 }}
          >
            {isSecure ? (
              <Lock size={16} style={{ color: 'var(--temp-text-muted)' }} />
            ) : (
              <Search size={16} style={{ color: 'var(--temp-text-muted)' }} />
            )}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Input field */}
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        placeholder={placeholder}
        className="flex-1 bg-transparent outline-none border-none"
        style={{
          color: 'var(--temp-text-primary)',
          fontSize: mode === 'ai' ? '1.25rem' : '0.9375rem',
          fontWeight: mode === 'ai' ? '500' : '400',
        }}
      />

      {/* Loading indicator */}
      <AnimatePresence>
        {isLoading && (
          <motion.div
            initial={{ opacity: 0, rotate: 0 }}
            animate={{ opacity: 1, rotate: 360 }}
            exit={{ opacity: 0 }}
            transition={{ 
              rotate: { duration: 1, repeat: Infinity, ease: "linear" } 
            }}
          >
            <RefreshCw size={16} style={{ color: 'var(--temp-text-muted)' }} />
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
