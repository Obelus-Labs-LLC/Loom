import { motion } from 'motion/react';
import { LoomAnimation } from './LoomAnimation';

interface ContentPaneProps {
  url: string;
  isLoading: boolean;
  mode: 'traditional' | 'ai';
}

export function ContentPane({ url, isLoading, mode }: ContentPaneProps) {
  // Mock content based on URL
  const renderContent = () => {
    if (isLoading) {
      return (
        <motion.div 
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="flex items-center justify-center h-full"
        >
          <div className="flex flex-col items-center gap-4">
            <LoomAnimation size={60} />
            <p 
              style={{ 
                fontFamily: 'var(--font-tension-sans)', 
                color: 'var(--temp-text-secondary)' 
              }}
            >
              Weaving content...
            </p>
          </div>
        </motion.div>
      );
    }

    if (!url || url === 'about:home') {
      return (
        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ type: "spring", stiffness: 200, damping: 20 }}
          className={`flex flex-col ${mode === 'ai' ? 'items-center justify-center' : ''} h-full p-8`}
        >
          <h1 
            style={{ 
              fontFamily: 'var(--font-weave-serif)', 
              color: 'var(--temp-text-primary)',
              fontSize: mode === 'ai' ? '3rem' : '2rem',
              marginBottom: mode === 'ai' ? '2rem' : '1rem',
            }}
          >
            Welcome to Loom
          </h1>
          <p 
            style={{ 
              fontFamily: 'var(--font-weave-serif)', 
              color: 'var(--temp-text-secondary)',
              fontSize: mode === 'ai' ? '1.25rem' : '1rem',
              maxWidth: mode === 'ai' ? '600px' : '800px',
              textAlign: mode === 'ai' ? 'center' : 'left',
              lineHeight: '1.6',
            }}
          >
            The native web navigator for FabricOS. Built on capability-security microkernel 
            with AI-native architecture. {mode === 'ai' ? 'Ask anything to begin your journey.' : ''}
          </p>
        </motion.div>
      );
    }

    // Sample article content
    return (
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ type: "spring", stiffness: 200, damping: 20 }}
        className={`${mode === 'ai' ? 'max-w-3xl mx-auto' : 'max-w-4xl'} p-8`}
      >
        <article>
          <h1 
            style={{ 
              fontFamily: 'var(--font-weave-serif)', 
              color: 'var(--temp-text-primary)',
              marginBottom: '1rem',
            }}
          >
            The Future of Secure Browsing
          </h1>
          
          <div 
            style={{ 
              fontFamily: 'var(--font-weave-serif)', 
              color: 'var(--temp-text-secondary)',
              fontSize: '0.875rem',
              marginBottom: '2rem',
            }}
          >
            March 4, 2026 · 5 min read
          </div>

          <div 
            style={{ 
              fontFamily: 'var(--font-weave-serif)', 
              color: 'var(--temp-text-primary)',
              lineHeight: '1.8',
              fontSize: '1.0625rem',
            }}
            className="space-y-4"
          >
            <p>
              In the evolving landscape of web navigation, security and user experience have often been at odds. 
              Traditional browsers prioritize compatibility and features, sometimes at the expense of fundamental security principles.
            </p>
            
            <p>
              Loom represents a paradigm shift—a browser built from the ground up on a capability-security microkernel. 
              Every action, every permission, every piece of data flows through a security model that assumes zero trust by default.
            </p>

            <h2 style={{ marginTop: '2rem', marginBottom: '1rem' }}>
              Capability-Based Security
            </h2>

            <p>
              Unlike traditional access control lists, capability-based security provides unforgeable tokens of authority. 
              A web page cannot access a resource unless it possesses the specific capability to do so. 
              This eliminates entire classes of vulnerabilities that plague modern browsers.
            </p>

            <p>
              The microkernel architecture ensures that the browser's core is minimal and verifiable. 
              Extensions, rendering engines, and network stacks all run in isolated compartments, 
              communicating only through well-defined capability channels.
            </p>

            <h2 style={{ marginTop: '2rem', marginBottom: '1rem' }}>
              AI-Native Architecture
            </h2>

            <p>
              But security alone isn't enough. Loom integrates AI at the architectural level, 
              not as an afterthought or chatbot overlay. The browser understands context, intent, and information flow. 
              It can summarize, extract, connect, and reason about the content you're viewing.
            </p>

            <p>
              This dual-mode approach—traditional for precise control, AI-assisted for exploration and discovery—
              gives users the best of both worlds. Dense information when you need it, generous whitespace when you don't.
            </p>

            <div 
              style={{ 
                marginTop: '3rem',
                padding: '1.5rem',
                background: 'var(--temp-surface)',
                clipPath: 'polygon(12px 0, calc(100% - 12px) 0, 100% 12px, 100% calc(100% - 12px), calc(100% - 12px) 100%, 12px 100%, 0 calc(100% - 12px), 0 12px)',
              }}
            >
              <p style={{ fontStyle: 'italic', margin: 0 }}>
                "The web isn't just pages to visit—it's threads to follow, patterns to discover, 
                and connections to make. Loom makes that tangible."
              </p>
            </div>

            <p style={{ marginTop: '2rem' }}>
              As we move forward into an era where AI and security must coexist harmoniously, 
              Loom shows what's possible when you start with the right foundations.
            </p>
          </div>
        </article>
      </motion.div>
    );
  };

  return (
    <motion.div 
      layout
      className="flex-1 overflow-y-auto"
      style={{ 
        background: mode === 'ai' ? 'var(--temp-bg-primary)' : 'var(--temp-bg-secondary)',
      }}
    >
      {renderContent()}
    </motion.div>
  );
}
