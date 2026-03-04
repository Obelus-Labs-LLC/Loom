import { motion } from 'motion/react';

interface ThreadPulseProps {
  isActive: boolean;
}

export function ThreadPulse({ isActive }: ThreadPulseProps) {
  if (!isActive) return null;

  return (
    <svg
      className="absolute inset-0 w-full h-full pointer-events-none"
      style={{ zIndex: 1 }}
    >
      {/* Animated thread traces from edge to center */}
      <motion.path
        d="M 0 50 Q 25 50 50 50"
        stroke="var(--temp-border)"
        strokeWidth="1"
        fill="none"
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: [0, 0.6, 0] }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />
      <motion.path
        d="M 100 50 Q 75 50 50 50"
        stroke="var(--temp-border)"
        strokeWidth="1"
        fill="none"
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: [0, 0.6, 0] }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
          delay: 0.5,
        }}
      />
      <motion.path
        d="M 50 0 Q 50 25 50 50"
        stroke="var(--temp-border)"
        strokeWidth="1"
        fill="none"
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: [0, 0.6, 0] }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
          delay: 1,
        }}
      />
      <motion.path
        d="M 50 100 Q 50 75 50 50"
        stroke="var(--temp-border)"
        strokeWidth="1"
        fill="none"
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: [0, 0.6, 0] }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
          delay: 1.5,
        }}
      />
    </svg>
  );
}
