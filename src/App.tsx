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
        <footer style={{ marginTop: 'auto', paddingTop: '2rem', textAlign: 'center', width: '100%', paddingBottom: '0.5rem' }}>
          <span style={{ fontSize: '0.85rem', color: '#888', display: 'block', lineHeight: '1.4' }}>
            Bu proje açık kaynaklıdır; dileyen herkes <a href="https://github.com/gokcehan0/Tamerlane-Chess-Engine" target="_blank" rel="noopener noreferrer" style={{ color: '#f59e0b', textDecoration: 'none', fontWeight: 600 }}>GitHub'da</a> katkıda bulunabilir. <br/>
            <span style={{ opacity: 0.7, fontSize: '0.75rem' }}>(Oyun verileri toplanmaktadır.)</span>
          </span>
        </footer>
      </div>
    </ThemeProvider>
  )
}

export default App
