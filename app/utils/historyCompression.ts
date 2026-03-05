// Loom Concierge - History Compression Utilities
// Token-saving strategies for long chat sessions

import type { Message } from '../store/concierge'

export interface CompressionResult {
  originalTokens: number
  compressedTokens: number
  compressionRatio: number
  messagesRemoved: number
  summary: string
}

/**
 * Summarize a batch of messages into a condensed form
 * This is a client-side placeholder - in production, this would call an API
 */
export async function summarizeMessages(
  messages: Message[],
  apiUrl?: string,
  apiKey?: string
): Promise<string> {
  if (messages.length === 0) return ''

  // If we have an API, use it for intelligent summarization
  if (apiUrl) {
    try {
      const response = await fetch(apiUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(apiKey && { 'Authorization': `Bearer ${apiKey}` })
        },
        body: JSON.stringify({
          model: 'gpt-4',
          messages: [
            {
              role: 'system',
              content: 'Summarize the following conversation into 2-3 sentences. Capture the main topics and conclusions.'
            },
            {
              role: 'user',
              content: messages.map(m => `${m.role}: ${m.content.substring(0, 200)}`).join('\n')
            }
          ],
          stream: false
        })
      })

      if (response.ok) {
        const data = await response.json()
        return data.choices?.[0]?.message?.content || ''
      }
    } catch (e) {
      console.warn('API summarization failed, using fallback:', e)
    }
  }

  // Fallback: Extractive summarization
  return extractiveSummary(messages)
}

/**
 * Create an extractive summary from messages
 * Picks key sentences and topics without API call
 */
function extractiveSummary(messages: Message[]): string {
  // Filter out system messages and already compressed
  const userAndAssistant = messages.filter(
    m => m.role !== 'system' && !m.compressed
  )

  if (userAndAssistant.length === 0) {
    return '[Previous conversation history]'
  }

  // Group by topic (simple heuristic: user messages start new topics)
  const topics: string[] = []
  let currentTopic = ''

  for (const msg of userAndAssistant) {
    if (msg.role === 'user') {
      if (currentTopic) topics.push(currentTopic)
      currentTopic = msg.content.substring(0, 100)
    }
  }
  if (currentTopic) topics.push(currentTopic)

  // Create summary
  const uniqueTopics = [...new Set(topics)].slice(0, 3)
  const messageCount = userAndAssistant.length
  
  return `[Conversation history: ${messageCount} messages covering: ${uniqueTopics.join('; ')}. Details available on request.]`
}

/**
 * Select which messages to keep vs compress
 * Strategy: Keep most recent N messages, compress older ones
 */
export function selectMessagesForCompression(
  messages: Message[],
  keepRecent: number = 4,
  maxBeforeCompression: number = 10
): {
  toKeep: Message[]
  toCompress: Message[]
  shouldCompress: boolean
} {
  // Don't compress short histories
  if (messages.length < maxBeforeCompression) {
    return {
      toKeep: messages,
      toCompress: [],
      shouldCompress: false
    }
  }

  // Split: keep recent N, compress the rest
  const toKeep = messages.slice(-keepRecent)
  const toCompress = messages.slice(0, -keepRecent)

  return {
    toKeep,
    toCompress,
    shouldCompress: true
  }
}

/**
 * Calculate token count for a message batch
 */
export function estimateTokenCount(messages: Message[]): number {
  return messages.reduce((sum, m) => {
    if (m.tokens) return sum + m.tokens
    // Rough estimate: ~4 chars per token
    return sum + Math.ceil(m.content.length / 4)
  }, 0)
}

/**
 * Compress history with intelligent batching
 */
export async function compressHistoryIntelligent(
  messages: Message[],
  apiUrl?: string,
  apiKey?: string
): Promise<{ compressed: Message[]; result: CompressionResult }> {
  const { toKeep, toCompress, shouldCompress } = selectMessagesForCompression(messages)

  if (!shouldCompress) {
    return {
      compressed: messages,
      result: {
        originalTokens: estimateTokenCount(messages),
        compressedTokens: estimateTokenCount(messages),
        compressionRatio: 1,
        messagesRemoved: 0,
        summary: ''
      }
    }
  }

  const originalTokens = estimateTokenCount(messages)
  
  // Generate summary
  const summary = await summarizeMessages(toCompress, apiUrl, apiKey)
  
  // Create compressed message
  const compressedMessage: Message = {
    id: `compressed-${Date.now()}`,
    role: 'system',
    content: summary,
    timestamp: Date.now(),
    compressed: true,
    tokens: Math.ceil(summary.length / 4)
  }

  const compressed = [compressedMessage, ...toKeep]
  const compressedTokens = estimateTokenCount(compressed)

  return {
    compressed,
    result: {
      originalTokens,
      compressedTokens,
      compressionRatio: originalTokens / compressedTokens,
      messagesRemoved: toCompress.length,
      summary
    }
  }
}

/**
 * Check if compression is recommended based on token count
 */
export function shouldCompressHistory(
  messages: Message[],
  tokenThreshold: number = 2000
): boolean {
  const tokenCount = estimateTokenCount(messages)
  return tokenCount > tokenThreshold
}

/**
 * Get compression recommendation for UI display
 */
export function getCompressionRecommendation(
  messages: Message[]
): {
  shouldCompress: boolean
  tokenCount: number
  messageCount: number
  estimatedSavings: number
} {
  const tokenCount = estimateTokenCount(messages)
  const messageCount = messages.length
  
  if (messageCount < 10) {
    return {
      shouldCompress: false,
      tokenCount,
      messageCount,
      estimatedSavings: 0
    }
  }

  // Estimate: older messages typically ~70% of tokens
  const estimatedSavings = Math.floor(tokenCount * 0.7)

  return {
    shouldCompress: tokenCount > 2000,
    tokenCount,
    messageCount,
    estimatedSavings
  }
}
