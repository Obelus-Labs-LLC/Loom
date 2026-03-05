// Loom Concierge Overlay - Fixed sidebar with streaming chat
// Pattern: NextChat-style floating assistant

import React, { useState, useRef, useEffect, useCallback } from 'react'
import { useConciergeStore, useMessages, useIsStreaming, useIsOpen, type SessionMask } from '../store/concierge'
import { useConciergeStream } from '../hooks/useConciergeStream'

interface ConciergeOverlayProps {
  apiUrl?: string
  apiKey?: string
  model?: string
}

export const ConciergeOverlay: React.FC<ConciergeOverlayProps> = ({
  apiUrl = '/api/chat',
  apiKey,
  model = 'gpt-4'
}) => {
  const [input, setInput] = useState('')
  const [showMasks, setShowMasks] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)
  
  // Store selectors
  const messages = useMessages()
  const isStreaming = useIsStreaming()
  const isOpen = useIsOpen()
  
  const {
    sessionMask,
    masks,
    isMinimized,
    toggleOpen,
    setMinimized,
    setSessionMask,
    clearMessages,
    compressHistory,
    getTokenEstimate
  } = useConciergeStore()

  // Streaming hook
  const { streamResponse, abortStream } = useConciergeStream({
    apiUrl,
    apiKey,
    model,
    onError: (error) => {
      console.error('Concierge error:', error)
      alert(`Error: ${error.message}`)
    }
  })

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, isStreaming])

  // Focus input when opened
  useEffect(() => {
    if (isOpen && !isMinimized) {
      inputRef.current?.focus()
    }
  }, [isOpen, isMinimized])

  // Handle send
  const handleSend = useCallback(async () => {
    if (!input.trim() || isStreaming) return
    
    const message = input.trim()
    setInput('')
    await streamResponse(message)
  }, [input, isStreaming, streamResponse])

  // Handle key press (Enter to send, Shift+Enter for newline)
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }, [handleSend])

  // Toggle mask selector
  const currentMask = masks.find(m => m.id === sessionMask) || masks[0]

  // Calculate token count
  const tokenCount = getTokenEstimate()
  const shouldCompress = tokenCount > 2000

  if (!isOpen) {
    return (
      <button
        onClick={toggleOpen}
        style={{
          position: 'fixed',
          right: '20px',
          bottom: '20px',
          width: '56px',
          height: '56px',
          borderRadius: '50%',
          background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
          border: 'none',
          cursor: 'pointer',
          zIndex: 1000,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '24px',
          boxShadow: '0 4px 12px rgba(99, 102, 241, 0.4)',
          transition: 'transform 0.2s, box-shadow 0.2s'
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.transform = 'scale(1.05)'
          e.currentTarget.style.boxShadow = '0 6px 20px rgba(99, 102, 241, 0.5)'
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.transform = 'scale(1)'
          e.currentTarget.style.boxShadow = '0 4px 12px rgba(99, 102, 241, 0.4)'
        }}
        aria-label="Open Concierge"
      >
        🤖
      </button>
    )
  }

  if (isMinimized) {
    return (
      <div
        style={{
          position: 'fixed',
          right: '20px',
          bottom: '20px',
          width: '300px',
          height: '48px',
          background: 'rgba(30, 30, 40, 0.95)',
          backdropFilter: 'blur(12px)',
          borderRadius: '12px',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          zIndex: 1000,
          display: 'flex',
          alignItems: 'center',
          padding: '0 16px',
          cursor: 'pointer',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4)'
        }}
        onClick={() => setMinimized(false)}
      >
        <span style={{ marginRight: '8px' }}>🤖</span>
        <span style={{ flex: 1, color: '#fff', fontSize: '14px', fontWeight: 500 }}>
          Concierge
        </span>
        <button
          onClick={(e) => {
            e.stopPropagation()
            toggleOpen()
          }}
          style={{
            background: 'none',
            border: 'none',
            color: '#888',
            cursor: 'pointer',
            fontSize: '18px',
            padding: '4px'
          }}
        >
          ×
        </button>
      </div>
    )
  }

  return (
    <div
      style={{
        position: 'fixed',
        right: '20px',
        bottom: '20px',
        width: '380px',
        height: '600px',
        maxHeight: 'calc(100vh - 40px)',
        background: 'rgba(30, 30, 40, 0.95)',
        backdropFilter: 'blur(12px)',
        WebkitBackdropFilter: 'blur(12px)',
        borderRadius: '16px',
        border: '1px solid rgba(255, 255, 255, 0.1)',
        zIndex: 1000,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4)'
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          padding: '16px',
          borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
          background: 'rgba(0, 0, 0, 0.2)'
        }}
      >
        <span style={{ fontSize: '20px', marginRight: '8px' }}>🤖</span>
        <div style={{ flex: 1 }}>
          <div style={{ color: '#fff', fontWeight: 600, fontSize: '14px' }}>
            Concierge
          </div>
          <div style={{ color: '#888', fontSize: '11px' }}>
            {currentMask.name}
          </div>
        </div>
        
        {/* Token counter */}
        <div
          style={{
            padding: '4px 8px',
            borderRadius: '4px',
            background: shouldCompress ? 'rgba(239, 68, 68, 0.2)' : 'rgba(99, 102, 241, 0.2)',
            color: shouldCompress ? '#ef4444' : '#818cf8',
            fontSize: '11px',
            marginRight: '8px',
            cursor: shouldCompress ? 'pointer' : 'default'
          }}
          onClick={() => shouldCompress && compressHistory()}
          title={shouldCompress ? 'Click to compress history' : `${tokenCount} tokens`}
        >
          {tokenCount > 1000 ? `${Math.round(tokenCount / 100) / 10}k` : tokenCount}
        </div>

        {/* Controls */}
        <button
          onClick={() => setShowMasks(!showMasks)}
          style={{
            background: 'none',
            border: 'none',
            color: '#888',
            cursor: 'pointer',
            fontSize: '16px',
            padding: '4px'
          }}
          title="Change persona"
        >
          🎭
        </button>
        <button
          onClick={() => setMinimized(true)}
          style={{
            background: 'none',
            border: 'none',
            color: '#888',
            cursor: 'pointer',
            fontSize: '16px',
            padding: '4px',
            marginLeft: '4px'
          }}
        >
          −
        </button>
        <button
          onClick={toggleOpen}
          style={{
            background: 'none',
            border: 'none',
            color: '#888',
            cursor: 'pointer',
            fontSize: '18px',
            padding: '4px',
            marginLeft: '4px'
          }}
        >
          ×
        </button>
      </div>

      {/* Mask selector */}
      {showMasks && (
        <div
          style={{
            padding: '8px',
            background: 'rgba(0, 0, 0, 0.3)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)'
          }}
        >
          {masks.map((mask) => (
            <button
              key={mask.id}
              onClick={() => {
                setSessionMask(mask.id)
                setShowMasks(false)
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                width: '100%',
                padding: '8px 12px',
                background: sessionMask === mask.id ? 'rgba(99, 102, 241, 0.2)' : 'transparent',
                border: 'none',
                borderRadius: '8px',
                cursor: 'pointer',
                color: sessionMask === mask.id ? '#818cf8' : '#ccc',
                fontSize: '13px',
                marginBottom: '4px'
              }}
            >
              <span style={{ marginRight: '8px' }}>{mask.icon}</span>
              {mask.name}
            </button>
          ))}
        </div>
      )}

      {/* Messages */}
      <div
        style={{
          flex: 1,
          overflow: 'auto',
          padding: '16px',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px'
        }}
      >
        {messages.length === 0 && (
          <div
            style={{
              textAlign: 'center',
              color: '#666',
              padding: '40px 20px'
            }}
          >
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>👋</div>
            <div style={{ fontSize: '14px', marginBottom: '8px' }}>
              Welcome to Concierge
            </div>
            <div style={{ fontSize: '12px' }}>
              Your AI assistant for browsing and tasks.
              <br />
              Try asking about the current page or any topic.
            </div>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            style={{
              alignSelf: msg.role === 'user' ? 'flex-end' : 'flex-start',
              maxWidth: '85%',
              padding: '12px 16px',
              borderRadius: msg.role === 'user' ? '16px 16px 4px 16px' : '16px 16px 16px 4px',
              background: msg.role === 'user' 
                ? 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' 
                : msg.compressed 
                  ? 'rgba(99, 102, 241, 0.1)'
                  : 'rgba(255, 255, 255, 0.08)',
              color: msg.role === 'user' ? '#fff' : '#e0e0e0',
              fontSize: '13px',
              lineHeight: '1.5',
              whiteSpace: 'pre-wrap',
              border: msg.compressed ? '1px dashed rgba(99, 102, 241, 0.3)' : 'none'
            }}
          >
            {msg.compressed && (
              <div style={{ fontSize: '10px', color: '#818cf8', marginBottom: '4px' }}>
                📦 Summarized
              </div>
            )}
            {msg.content}
          </div>
        ))}

        {/* Streaming indicator */}
        {isStreaming && (
          <div
            style={{
              alignSelf: 'flex-start',
              padding: '12px 16px',
              borderRadius: '16px 16px 16px 4px',
              background: 'rgba(255, 255, 255, 0.08)',
              color: '#e0e0e0',
              fontSize: '13px'
            }}
          >
            <span style={{ animation: 'pulse 1s infinite' }}>●</span>
            <span style={{ animation: 'pulse 1s infinite 0.2s' }}>●</span>
            <span style={{ animation: 'pulse 1s infinite 0.4s' }}>●</span>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div
        style={{
          padding: '16px',
          borderTop: '1px solid rgba(255, 255, 255, 0.1)',
          background: 'rgba(0, 0, 0, 0.2)'
        }}
      >
        <div style={{ display: 'flex', gap: '8px' }}>
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ask anything..."
            rows={1}
            style={{
              flex: 1,
              padding: '12px 16px',
              borderRadius: '24px',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              background: 'rgba(255, 255, 255, 0.05)',
              color: '#fff',
              fontSize: '13px',
              resize: 'none',
              outline: 'none',
              minHeight: '44px',
              maxHeight: '120px'
            }}
          />
          <button
            onClick={isStreaming ? abortStream : handleSend}
            style={{
              width: '44px',
              height: '44px',
              borderRadius: '50%',
              border: 'none',
              background: isStreaming 
                ? 'rgba(239, 68, 68, 0.8)' 
                : 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
              color: '#fff',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '16px',
              opacity: input.trim() || isStreaming ? 1 : 0.5
            }}
            disabled={!input.trim() && !isStreaming}
          >
            {isStreaming ? '⏹' : '➤'}
          </button>
        </div>

        {/* Footer actions */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            marginTop: '8px',
            fontSize: '11px',
            color: '#666'
          }}
        >
          <button
            onClick={clearMessages}
            style={{
              background: 'none',
              border: 'none',
              color: '#666',
              cursor: 'pointer',
              fontSize: '11px'
            }}
          >
            Clear chat
          </button>
          <span>Shift + Enter for newline</span>
        </div>
      </div>

      {/* CSS animations */}
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  )
}

export default ConciergeOverlay
