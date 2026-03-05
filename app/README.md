# Loom App - AI-Native Interface

React-based application layer for the Loom browser's AI-Native mode.

## Architecture

```
app/
├── store/          # Zustand state management (local-first)
├── hooks/          # React hooks for streaming, API calls
├── components/     # React components (Concierge overlay)
├── utils/          # Helper functions (compression, token counting)
└── package.json    # App-level dependencies
```

## Concierge Overlay

The Concierge is a floating AI assistant panel:

### Features
- **Zustand + localStorage**: Persistent chat history
- **Streaming responses**: Fetch API with ReadableStream
- **Session masks**: Persona switching (Developer, System Admin, etc.)
- **History compression**: Token-saving for long sessions
- **Glassmorphism UI**: Blur backdrop, gradient accents

### Usage

```tsx
import { ConciergeOverlay } from '@loom/app'

function App() {
  return (
    <>
      <Browser />
      <ConciergeOverlay 
        apiUrl="/api/chat"
        apiKey={process.env.OPENAI_KEY}
        model="gpt-4"
      />
    </>
  )
}
```

### State Management

```tsx
import { useConciergeStore } from '@loom/app'

const messages = useConciergeStore((state) => state.messages)
const addMessage = useConciergeStore((state) => state.addMessage)
const compressHistory = useConciergeStore((state) => state.compressHistory)
```

### Streaming Hook

```tsx
import { useConciergeStream } from '@loom/app'

const { streamResponse, abortStream } = useConciergeStream({
  apiUrl: '/api/chat',
  onError: (e) => console.error(e),
  onComplete: () => console.log('Done')
})

// Start streaming
await streamResponse('Hello, can you help me?')

// Cancel
abortStream()
```

## Session Masks

Default personas:
- **General Assistant** (✨): Default helpful assistant
- **System Administrator** (⚡): Technical guidance, troubleshooting
- **Code Assistant** (💻): Software development help
- **Research Assistant** (🔍): Information finding, summarization

## History Compression

Automatic compression when token count exceeds threshold:

```tsx
// Manual compression
await compressHistory()

// Check if needed
const tokenCount = getTokenEstimate()
if (tokenCount > 2000) {
  await compressHistory()
}
```

## Styling

The Concierge uses inline styles with CSS-in-JS pattern:
- Glassmorphism: `backdrop-filter: blur(12px)`
- Gradient accents: `linear-gradient(135deg, #6366f1, #8b5cf6)`
- Dark theme optimized for Loom's AI-Native mode

## Dependencies

- `zustand`: State management with persistence
- `react`: UI components
- No external CSS framework (self-contained styles)
