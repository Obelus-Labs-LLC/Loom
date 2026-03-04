import { motion } from 'motion/react';

interface LoomAnimationProps {
  size?: number;
}

export function LoomAnimation({ size = 40 }: LoomAnimationProps) {
  return (
    <div
      className="relative flex items-center justify-center"
      style={{ width: size, height: size }}
    >
      {/* Interlacing threads animation */}
      {[0, 1, 2, 3].map((index) => (
        <motion.div
          key={index}
          className="absolute"
          style={{
            width: '60%',
            height: '2px',
            background: 'var(--temp-text-secondary)',
            transformOrigin: 'center',
          }}
          animate={{
            rotate: [0, 90, 180, 270, 360],
            opacity: [0.3, 0.8, 0.3],
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: "easeInOut",
            delay: index * 0.2,
          }}
        />
      ))}
      <motion.div
        className="absolute"
        style={{
          width: '50%',
          height: '50%',
          border: '2px solid var(--temp-text-secondary)',
          borderRadius: '50%',
        }}
        animate={{
          scale: [0.8, 1.2, 0.8],
          opacity: [0.3, 0.6, 0.3],
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />
    </div>
  );
}
