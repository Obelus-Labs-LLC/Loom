import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { TabBar } from './components/TabBar';
import { AddressBar } from './components/AddressBar';
import { NavigationToolbar } from './components/NavigationToolbar';
import { ContentPane } from './components/ContentPane';
import { SettingsPanel } from './components/SettingsPanel';
import { ThreadPulse } from './components/ThreadPulse';

interface Tab {
  id: string;
  title: string;
  url: string;
  isActive: boolean;
}

type Temperature = 'warm' | 'cool' | 'neutral' | 'auto';
type Mode = 'traditional' | 'ai';

export default function App() {
  const [tabs, setTabs] = useState<Tab[]>([
    { id: '1', title: 'Home', url: 'about:home', isActive: true },
  ]);
  const [currentUrl, setCurrentUrl] = useState('about:home');
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [history, setHistory] = useState<string[]>(['about:home']);
  const [historyIndex, setHistoryIndex] = useState(0);
  
  const [temperature, setTemperature] = useState<Temperature>('auto');
  const [globalMode, setGlobalMode] = useState<Mode>('traditional');
  const [windowModeOverride, setWindowModeOverride] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // Determine current mode (override or global)
  const currentMode: Mode = windowModeOverride 
    ? (globalMode === 'traditional' ? 'ai' : 'traditional')
    : globalMode;

  // Auto temperature based on time of day
  useEffect(() => {
    if (temperature === 'auto') {
      const hour = new Date().getHours();
      let autoTemp: 'warm' | 'cool' | 'neutral';
      
      if (hour >= 6 && hour < 12) {
        autoTemp = 'warm'; // Morning
      } else if (hour >= 12 && hour < 18) {
        autoTemp = 'neutral'; // Afternoon
      } else {
        autoTemp = 'cool'; // Evening/Night
      }
      
      applyTemperature(autoTemp);
    } else {
      applyTemperature(temperature);
    }
  }, [temperature]);

  const applyTemperature = (temp: 'warm' | 'cool' | 'neutral') => {
    const root = document.documentElement;
    
    if (temp === 'warm') {
      root.style.setProperty('--temp-bg-primary', 'var(--temp-warm-50)');
      root.style.setProperty('--temp-bg-secondary', 'var(--temp-warm-100)');
      root.style.setProperty('--temp-surface', 'var(--temp-warm-200)');
      root.style.setProperty('--temp-border', 'var(--temp-warm-300)');
    } else if (temp === 'cool') {
      root.style.setProperty('--temp-bg-primary', 'var(--temp-cool-50)');
      root.style.setProperty('--temp-bg-secondary', 'var(--temp-cool-100)');
      root.style.setProperty('--temp-surface', 'var(--temp-cool-200)');
      root.style.setProperty('--temp-border', 'var(--temp-cool-300)');
    } else {
      root.style.setProperty('--temp-bg-primary', 'var(--temp-neutral-50)');
      root.style.setProperty('--temp-bg-secondary', 'var(--temp-neutral-100)');
      root.style.setProperty('--temp-surface', 'var(--temp-neutral-200)');
      root.style.setProperty('--temp-border', 'var(--temp-neutral-300)');
    }
  };

  const activeTab = tabs.find(tab => tab.isActive);

  const handleTabClick = (id: string) => {
    setTabs(tabs.map(tab => ({ ...tab, isActive: tab.id === id })));
    const tab = tabs.find(t => t.id === id);
    if (tab) {
      setCurrentUrl(tab.url);
      setInputValue(tab.url);
    }
  };

  const handleTabClose = (id: string) => {
    if (tabs.length === 1) return;
    
    const tabIndex = tabs.findIndex(tab => tab.id === id);
    const newTabs = tabs.filter(tab => tab.id !== id);
    
    if (tabs[tabIndex].isActive && newTabs.length > 0) {
      const newActiveIndex = Math.min(tabIndex, newTabs.length - 1);
      newTabs[newActiveIndex].isActive = true;
      setCurrentUrl(newTabs[newActiveIndex].url);
      setInputValue(newTabs[newActiveIndex].url);
    }
    
    setTabs(newTabs);
  };

  const handleNewTab = () => {
    const newTab: Tab = {
      id: Date.now().toString(),
      title: 'New Tab',
      url: 'about:home',
      isActive: true,
    };
    setTabs(tabs.map(tab => ({ ...tab, isActive: false })).concat(newTab));
    setCurrentUrl('about:home');
    setInputValue('');
  };

  const handleNavigate = (url: string) => {
    setIsLoading(true);
    
    // Simulate page load with tension animation
    setTimeout(() => {
      setCurrentUrl(url);
      setHistory(prev => [...prev.slice(0, historyIndex + 1), url]);
      setHistoryIndex(prev => prev + 1);
      
      setTabs(tabs.map(tab => 
        tab.isActive 
          ? { ...tab, title: url === 'about:home' ? 'Home' : url, url }
          : tab
      ));
      
      setIsLoading(false);
    }, 800);
  };

  const handleBack = () => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      const url = history[newIndex];
      setCurrentUrl(url);
      setInputValue(url);
      
      setTabs(tabs.map(tab => 
        tab.isActive 
          ? { ...tab, url }
          : tab
      ));
    }
  };

  const handleForward = () => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      const url = history[newIndex];
      setCurrentUrl(url);
      setInputValue(url);
      
      setTabs(tabs.map(tab => 
        tab.isActive 
          ? { ...tab, url }
          : tab
      ));
    }
  };

  const handleRefresh = () => {
    setIsLoading(true);
    setTimeout(() => setIsLoading(false), 500);
  };

  const handleHome = () => {
    handleNavigate('about:home');
    setInputValue('');
  };

  return (
    <div 
      className="w-screen h-screen flex flex-col overflow-hidden"
      style={{ 
        fontFamily: 'var(--font-tension-sans)',
        background: 'var(--temp-bg-primary)',
      }}
    >
      {/* Mode transition wrapper - Unweave/Reweave animation */}
      <AnimatePresence mode="wait">
        <motion.div
          key={currentMode}
          initial={{ opacity: 0, scale: 0.98 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 1.02 }}
          transition={{
            type: "spring",
            stiffness: 200,
            damping: 25,
            mass: 1,
          }}
          className="flex flex-col h-full relative"
        >
          {currentMode === 'traditional' ? (
            // Traditional Mode - Dense layout
            <>
              {/* Tab Bar */}
              <TabBar
                tabs={tabs}
                onTabClick={handleTabClick}
                onTabClose={handleTabClose}
                onNewTab={handleNewTab}
              />

              {/* Navigation Toolbar + Address Bar */}
              <div 
                className="flex items-center gap-2 px-2 py-2"
                style={{ background: 'var(--temp-bg-secondary)' }}
              >
                <NavigationToolbar
                  onBack={handleBack}
                  onForward={handleForward}
                  onRefresh={handleRefresh}
                  onHome={handleHome}
                  onMenu={() => {}}
                  onSettings={() => setIsSettingsOpen(true)}
                  canGoBack={historyIndex > 0}
                  canGoForward={historyIndex < history.length - 1}
                />
                
                <motion.button
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={() => setGlobalMode('ai')}
                  className="px-3 py-1.5 text-sm ml-auto"
                  style={{
                    color: 'var(--temp-text-secondary)',
                    clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)',
                  }}
                >
                  Switch to AI
                </motion.button>
              </div>

              <div className="px-4 py-2" style={{ background: 'var(--temp-bg-secondary)' }}>
                <AddressBar
                  value={inputValue}
                  onChange={setInputValue}
                  onNavigate={handleNavigate}
                  isSecure={currentUrl.startsWith('https')}
                  isLoading={isLoading}
                  mode="traditional"
                />
              </div>

              {/* Content */}
              <ContentPane 
                url={currentUrl} 
                isLoading={isLoading} 
                mode="traditional"
              />

              {/* Window Override Indicator */}
              {windowModeOverride && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
                >
                  <ThreadPulse isActive={true} />
                </motion.div>
              )}
            </>
          ) : (
            // AI-Assisted Mode - Generous whitespace, floating input
            <>
              {/* Minimal top bar */}
              <div 
                className="flex items-center justify-between px-4 py-2"
                style={{ background: 'var(--temp-bg-primary)' }}
              >
                <div 
                  className="text-lg tracking-wide"
                  style={{ 
                    fontFamily: 'var(--font-weave-serif)',
                    color: 'var(--temp-text-secondary)',
                  }}
                >
                  Loom
                </div>
                
                <div className="flex items-center gap-4">
                  <motion.button
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    onClick={() => setGlobalMode(globalMode === 'ai' ? 'traditional' : 'ai')}
                    className="px-3 py-1.5 text-sm"
                    style={{
                      color: 'var(--temp-text-secondary)',
                      clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)',
                    }}
                  >
                    Switch to Traditional
                  </motion.button>
                  
                  <motion.button
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    onClick={() => setIsSettingsOpen(true)}
                    className="px-3 py-1.5 text-sm"
                    style={{
                      color: 'var(--temp-text-secondary)',
                      clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)',
                    }}
                  >
                    Settings
                  </motion.button>
                </div>
              </div>

              {/* Floating centered input */}
              <div className="flex-1 flex flex-col items-center justify-start pt-32 px-8">
                <motion.div
                  initial={{ y: 20, opacity: 0 }}
                  animate={{ y: 0, opacity: 1 }}
                  transition={{ 
                    type: "spring",
                    stiffness: 150,
                    damping: 20,
                    delay: 0.1,
                  }}
                  className="w-full max-w-3xl"
                >
                  <AddressBar
                    value={inputValue}
                    onChange={setInputValue}
                    onNavigate={handleNavigate}
                    isLoading={isLoading}
                    mode="ai"
                  />
                </motion.div>

                {/* Dimmed content below */}
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 0.6 }}
                  transition={{ delay: 0.2 }}
                  className="mt-12 w-full max-w-4xl"
                >
                  <ContentPane 
                    url={currentUrl} 
                    isLoading={isLoading} 
                    mode="ai"
                  />
                </motion.div>
              </div>

              {/* Window Override Indicator */}
              {windowModeOverride && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
                >
                  <ThreadPulse isActive={true} />
                </motion.div>
              )}

              {/* Minimal tab indicators at bottom */}
              {tabs.length > 1 && (
                <motion.div
                  initial={{ y: 20, opacity: 0 }}
                  animate={{ y: 0, opacity: 1 }}
                  className="flex items-center justify-center gap-2 p-4"
                >
                  {tabs.map((tab) => (
                    <motion.button
                      key={tab.id}
                      whileHover={{ scale: 1.2 }}
                      whileTap={{ scale: 0.9 }}
                      onClick={() => handleTabClick(tab.id)}
                      className="w-2 h-2 rounded-full transition-colors"
                      style={{
                        background: tab.isActive 
                          ? 'var(--temp-text-primary)' 
                          : 'var(--temp-text-muted)',
                      }}
                    />
                  ))}
                </motion.div>
              )}
            </>
          )}
        </motion.div>
      </AnimatePresence>

      {/* Settings Panel */}
      <SettingsPanel
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        temperature={temperature}
        onTemperatureChange={setTemperature}
        globalMode={globalMode}
        onGlobalModeChange={setGlobalMode}
        windowModeOverride={windowModeOverride}
        onWindowModeOverrideToggle={() => setWindowModeOverride(!windowModeOverride)}
      />
    </div>
  );
}