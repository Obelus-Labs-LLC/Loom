// Loom Concierge Streaming Hook
// Handles Fetch API with ReadableStream for streaming responses

import { useCallback, useRef } from 'react'
import { useConciergeStore, type Message } from '../store/concierge'

interface StreamOptions {
  apiUrl?: string
  apiKey?: string
  model?: string
  onError?: (error: Error) => void
  onComplete?: () => void
}

interface StreamController {
  abort: () => void
  isActive: () => boolean
}

export function useConciergeStream(options: StreamOptions = {}) {
  const abortControllerRef = useRef<AbortController | null>(null)
  
  const { 
    messages, 
    sessionMask, 
    masks,
    startStream, 
    endStream, 
    appendStream,
    addMessage 
  } = useConciergeStore()

  const getSystemPrompt = useCallback(() => {
    const mask = masks.find(m => m.id === sessionMask)
    return mask?.prompt || 'You are a helpful assistant.'
  }, [masks, sessionMask])

  const streamResponse = useCallback(async (
    userMessage: string
  ): Promise<StreamController> => {
    // Add user message to store
    addMessage({
      role: 'user',
      content: userMessage
    })

    // Start streaming state
    startStream()

    // Create abort controller for cancellation
    abortControllerRef.current = new AbortController()
    const { signal } = abortControllerRef.current

    const controller: StreamController = {
      abort: () => {
        abortControllerRef.current?.abort()
        endStream()
      },
      isActive: () => !signal.aborted
    }

    try {
      // Prepare messages for API
      const systemPrompt = getSystemPrompt()
      const apiMessages = [
        { role: 'system', content: systemPrompt },
        ...messages.map(m => ({
          role: m.role,
          content: m.compressed 
            ? `[Context: ${m.content}]` 
            : m.content
        })),
        { role: 'user', content: userMessage }
      ]

      // Make streaming request
      const response = await fetch(options.apiUrl || '/api/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(options.apiKey && { 'Authorization': `Bearer ${options.apiKey}` })
        },
        body: JSON.stringify({
          model: options.model || 'gpt-4',
          messages: apiMessages,
          stream: true
        }),
        signal
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      if (!response.body) {
        throw new Error('No response body')
      }

      // Read the stream
      const reader = response.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        
        if (done) {
          break
        }

        // Decode chunk
        buffer += decoder.decode(value, { stream: true })
        
        // Process SSE lines
        const lines = buffer.split('\n')
        buffer = lines.pop() || '' // Keep incomplete line in buffer

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6)
            
            // Check for stream end
            if (data === '[DONE]') {
              break
            }

            try {
              const parsed = JSON.parse(data)
              const content = parsed.choices?.[0]?.delta?.content || ''
              
              if (content) {
                appendStream(content)
              }
            } catch (e) {
              // Ignore parse errors for incomplete chunks
            }
          }
        }
      }

      // Process any remaining data
      if (buffer) {
        const line = buffer.trim()
        if (line.startsWith('data: ') && line !== 'data: [DONE]') {
          try {
            const data = line.slice(6)
            const parsed = JSON.parse(data)
            const content = parsed.choices?.[0]?.delta?.content || ''
            if (content) {
              appendStream(content)
            }
          } catch (e) {
            // Ignore parse errors
          }
        }
      }

      endStream()
      options.onComplete?.()

    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        console.log('Stream aborted')
      } else {
        console.error('Streaming error:', error)
        options.onError?.(error instanceof Error ? error : new Error(String(error)))
      }
      endStream()
    }

    return controller
  }, [messages, sessionMask, masks, getSystemPrompt, startStream, endStream, appendStream, addMessage, options])

  const abortStream = useCallback(() => {
    abortControllerRef.current?.abort()
    endStream()
  }, [endStream])

  return {
    streamResponse,
    abortStream
  }
}

// Simple non-streaming version for quick queries
export async function sendConciergeMessage(
  messages: Message[],
  userMessage: string,
  systemPrompt: string,
  options: StreamOptions = {}
): Promise<string> {
  const apiMessages = [
    { role: 'system', content: systemPrompt },
    ...messages.map(m => ({
      role: m.role,
      content: m.compressed ? `[Context: ${m.content}]` : m.content
    })),
    { role: 'user', content: userMessage }
  ]

  const response = await fetch(options.apiUrl || '/api/chat', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(options.apiKey && { 'Authorization': `Bearer ${options.apiKey}` })
    },
    body: JSON.stringify({
      model: options.model || 'gpt-4',
      messages: apiMessages,
      stream: false
    })
  })

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`)
  }

  const data = await response.json()
  return data.choices?.[0]?.message?.content || ''
}
