// Loom Concierge Store - Zustand + localStorage for local-first chat state
// Pattern: NextChat-style session management with compression

import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface Message {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  model?: string
  tokens?: number
  compressed?: boolean  // Marker for summarized content
}

export interface SessionMask {
  id: string
  name: string
  prompt: string
  icon?: string
}

interface ConciergeState {
  // Messages
  messages: Message[]
  isStreaming: boolean
  currentStream: string
  
  // Session configuration
  sessionMask: string
  masks: SessionMask[]
  
  // UI State
  isOpen: boolean
  isMinimized: boolean
  
  // Actions
  addMessage: (msg: Omit<Message, 'id' | 'timestamp'>) => void
  updateLastMessage: (content: string) => void
  appendStream: (chunk: string) => void
  startStream: () => void
  endStream: () => void
  clearMessages: () => void
  
  // History management
  compressHistory: () => Promise<void>
  getTokenEstimate: () => number
  
  // Session mask
  setSessionMask: (maskId: string) => void
  addMask: (mask: Omit<SessionMask, 'id'>) => void
  
  // UI
  toggleOpen: () => void
  setMinimized: (minimized: boolean) => void
}

// Generate unique IDs
const generateId = () => `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

// Default session masks
const DEFAULT_MASKS: SessionMask[] = [
  {
    id: 'default',
    name: 'General Assistant',
    prompt: 'You are a helpful assistant.',
    icon: '✨'
  },
  {
    id: 'system-admin',
    name: 'System Administrator',
    prompt: 'Act as a system administrator. Provide technical guidance, troubleshooting steps, and system configuration advice. Be precise and security-conscious.',
    icon: '⚡'
  },
  {
    id: 'developer',
    name: 'Code Assistant',
    prompt: 'You are an expert software developer. Provide clean, efficient code with explanations. Follow best practices and modern patterns.',
    icon: '💻'
  },
  {
    id: 'researcher',
    name: 'Research Assistant',
    prompt: 'Act as a research assistant. Help find information, summarize articles, and provide structured analysis of topics.',
    icon: '🔍'
  }
]

// Token estimation (rough approximation)
const estimateTokens = (text: string): number => {
  // Rough estimate: ~4 characters per token for English
  return Math.ceil(text.length / 4)
}

export const useConciergeStore = create<ConciergeState>()(
  persist(
    (set, get) => ({
      // Initial state
      messages: [],
      isStreaming: false,
      currentStream: '',
      sessionMask: 'default',
      masks: DEFAULT_MASKS,
      isOpen: false,
      isMinimized: false,

      // Add a new message
      addMessage: (msg) => {
        const message: Message = {
          ...msg,
          id: generateId(),
          timestamp: Date.now(),
          tokens: estimateTokens(msg.content)
        }
        set({ messages: [...get().messages, message] })
      },

      // Update the last assistant message (for streaming)
      updateLastMessage: (content) => {
        const messages = get().messages
        if (messages.length === 0) return
        
        const lastMsg = messages[messages.length - 1]
        if (lastMsg.role === 'assistant') {
          messages[messages.length - 1] = {
            ...lastMsg,
            content,
            tokens: estimateTokens(content)
          }
          set({ messages: [...messages] })
        }
      },

      // Streaming handlers
      startStream: () => set({ 
        isStreaming: true, 
        currentStream: '',
        // Add placeholder message for assistant
        messages: [...get().messages, {
          id: generateId(),
          role: 'assistant',
          content: '',
          timestamp: Date.now()
        }]
      }),

      appendStream: (chunk) => {
        const newStream = get().currentStream + chunk
        set({ currentStream: newStream })
        get().updateLastMessage(newStream)
      },

      endStream: () => set({ 
        isStreaming: false, 
        currentStream: '' 
      }),

      // Clear all messages
      clearMessages: () => set({ messages: [] }),

      // Compress history to save tokens
      compressHistory: async () => {
        const messages = get().messages
        if (messages.length < 10) return // Don't compress short histories

        // Find messages to compress (keep last 4, compress older)
        const keepCount = 4
        const toCompress = messages.slice(0, -keepCount)
        const toKeep = messages.slice(-keepCount)

        // Create summary of compressed messages
        const summaryContent = toCompress
          .filter(m => !m.compressed)
          .map(m => `${m.role}: ${m.content.substring(0, 100)}${m.content.length > 100 ? '...' : ''}`)
          .join('\n')

        if (summaryContent.length === 0) return

        // In real implementation, this would call an API to summarize
        // For now, create a compressed placeholder
        const compressed: Message = {
          id: generateId(),
          role: 'system',
          content: `[Previous conversation summarized: ${toCompress.length} messages covering various topics. Key points available on request.]`,
          timestamp: Date.now(),
          compressed: true,
          tokens: estimateTokens(summaryContent) / 10 // Estimated 10x compression
        }

        set({ messages: [compressed, ...toKeep] })
      },

      // Get total token estimate
      getTokenEstimate: () => {
        return get().messages.reduce((sum, m) => sum + (m.tokens || 0), 0)
      },

      // Session mask management
      setSessionMask: (maskId) => set({ sessionMask: maskId }),

      addMask: (mask) => {
        const newMask: SessionMask = {
          ...mask,
          id: generateId()
        }
        set({ masks: [...get().masks, newMask] })
      },

      // UI state
      toggleOpen: () => set({ isOpen: !get().isOpen }),
      setMinimized: (minimized) => set({ isMinimized: minimized })
    }),
    {
      name: 'concierge-storage',
      storage: createJSONStorage(() => localStorage),
      // Only persist messages and session config, not UI state
      partialize: (state) => ({
        messages: state.messages,
        sessionMask: state.sessionMask,
        masks: state.masks
      })
    }
  )
)

// Selector hooks for performance
export const useMessages = () => useConciergeStore((state) => state.messages)
export const useIsStreaming = () => useConciergeStore((state) => state.isStreaming)
export const useCurrentStream = () => useConciergeStore((state) => state.currentStream)
export const useSessionMask = () => useConciergeStore((state) => state.sessionMask)
export const useIsOpen = () => useConciergeStore((state) => state.isOpen)
