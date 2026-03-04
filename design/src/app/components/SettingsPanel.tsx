import { motion, AnimatePresence } from 'motion/react';
import { X, Thermometer, Moon, Sun, Waves } from 'lucide-react';
import { ThreadPulse } from './ThreadPulse';

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  temperature: 'warm' | 'cool' | 'neutral' | 'auto';
  onTemperatureChange: (temp: 'warm' | 'cool' | 'neutral' | 'auto') => void;
  globalMode: 'traditional' | 'ai';
  onGlobalModeChange: (mode: 'traditional' | 'ai') => void;
  windowModeOverride: boolean;
  onWindowModeOverrideToggle: () => void;
}

export function SettingsPanel({
  isOpen,
  onClose,
  temperature,
  onTemperatureChange,
  globalMode,
  onGlobalModeChange,
  windowModeOverride,
  onWindowModeOverrideToggle,
}: SettingsPanelProps) {
  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 bg-black/30 backdrop-blur-sm z-40"
          />

          {/* Panel */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: -20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: -20 }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
            className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg z-50 p-6"
            style={{
              background: 'var(--temp-bg-primary)',
              clipPath: 'polygon(20px 0, calc(100% - 20px) 0, 100% 20px, 100% calc(100% - 20px), calc(100% - 20px) 100%, 20px 100%, 0 calc(100% - 20px), 0 20px)',
              fontFamily: 'var(--font-tension-sans)',
              boxShadow: '0 20px 50px rgba(0, 0, 0, 0.15)',
            }}
          >
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
              <h2 
                style={{ 
                  color: 'var(--temp-text-primary)',
                  fontSize: '1.5rem',
                }}
              >
                Browser Settings
              </h2>
              <motion.button
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.9 }}
                onClick={onClose}
                className="p-2 hover:bg-black/5 transition-colors"
                style={{
                  clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)',
                }}
              >
                <X size={20} style={{ color: 'var(--temp-text-primary)' }} />
              </motion.button>
            </div>

            {/* Chromatic Temperature */}
            <div className="mb-8">
              <h3 
                className="mb-3"
                style={{ color: 'var(--temp-text-primary)' }}
              >
                Chromatic Temperature
              </h3>
              <p 
                className="mb-4 text-sm"
                style={{ color: 'var(--temp-text-secondary)' }}
              >
                Color responds to time of day and cognitive load. Never tied to function.
              </p>
              
              <div className="grid grid-cols-2 gap-2">
                {[
                  { value: 'auto' as const, label: 'Auto', icon: Waves },
                  { value: 'warm' as const, label: 'Warm', icon: Sun },
                  { value: 'cool' as const, label: 'Cool', icon: Moon },
                  { value: 'neutral' as const, label: 'Neutral', icon: Thermometer },
                ].map((option) => (
                  <motion.button
                    key={option.value}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={() => onTemperatureChange(option.value)}
                    className="flex items-center gap-2 p-3 transition-colors"
                    style={{
                      background: temperature === option.value 
                        ? 'var(--temp-surface)' 
                        : 'var(--temp-bg-secondary)',
                      color: 'var(--temp-text-primary)',
                      clipPath: 'polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px)',
                      border: temperature === option.value ? '2px solid var(--temp-border)' : 'none',
                    }}
                  >
                    <option.icon size={18} />
                    <span>{option.label}</span>
                  </motion.button>
                ))}
              </div>
            </div>

            {/* Mode Settings */}
            <div className="mb-8">
              <h3 
                className="mb-3"
                style={{ color: 'var(--temp-text-primary)' }}
              >
                Global Mode
              </h3>
              <p 
                className="mb-4 text-sm"
                style={{ color: 'var(--temp-text-secondary)' }}
              >
                Mode as structure, not color. Traditional is dense information. AI-assisted is generous whitespace.
              </p>
              
              <div className="flex gap-2">
                {[
                  { value: 'traditional' as const, label: 'Traditional' },
                  { value: 'ai' as const, label: 'AI-Assisted' },
                ].map((option) => (
                  <motion.button
                    key={option.value}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={() => onGlobalModeChange(option.value)}
                    className="flex-1 p-3 transition-colors"
                    style={{
                      background: globalMode === option.value 
                        ? 'var(--temp-surface)' 
                        : 'var(--temp-bg-secondary)',
                      color: 'var(--temp-text-primary)',
                      clipPath: 'polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px)',
                      border: globalMode === option.value ? '2px solid var(--temp-border)' : 'none',
                    }}
                  >
                    {option.label}
                  </motion.button>
                ))}
              </div>
            </div>

            {/* Window Override */}
            <div className="relative">
              <h3 
                className="mb-3"
                style={{ color: 'var(--temp-text-primary)' }}
              >
                This Window Override
              </h3>
              <p 
                className="mb-4 text-sm"
                style={{ color: 'var(--temp-text-secondary)' }}
              >
                Use a different mode for this window only.
              </p>
              
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={onWindowModeOverrideToggle}
                className="relative w-full p-4 transition-colors overflow-hidden"
                style={{
                  background: windowModeOverride 
                    ? 'var(--temp-surface)' 
                    : 'var(--temp-bg-secondary)',
                  color: 'var(--temp-text-primary)',
                  clipPath: 'polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px)',
                  border: windowModeOverride ? '2px solid var(--temp-border)' : 'none',
                }}
              >
                {windowModeOverride && <ThreadPulse isActive={true} />}
                <span className="relative z-10">
                  {windowModeOverride ? 'Override Active' : 'Use Global Mode'}
                </span>
              </motion.button>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
