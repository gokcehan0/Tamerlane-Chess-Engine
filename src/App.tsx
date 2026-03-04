import Game from './components/Game'
import { ThemeProvider } from './contexts/ThemeContext'
import './index.css'

function App() {
  return (
    <ThemeProvider>
      <div style={{ 
        minHeight: '100vh', 
        background: '#0f0f0f', 
        color: '#eee',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: '1rem'
      }}>
        <h1 style={{ 
          fontSize: '1.5rem', 
          marginBottom: '1rem',
          background: 'linear-gradient(135deg, #f59e0b, #ef4444)',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          fontWeight: 'bold'
        }}>
          ♛ Tamerlane Chess Bot
        </h1>
        <div style={{ width: '100%', maxWidth: '1200px' }}>
          <Game />
        </div>
      </div>
    </ThemeProvider>
  )
}

export default App
