import { listen } from '@tauri-apps/api/event'
import './overlay.css'

const el = document.getElementById('overlay')
const label = document.getElementById('label')

function show(text) {
  label.textContent = text || 'DPI'
  el.classList.remove('show')
  // restart animation
  void el.offsetWidth
  el.classList.add('show')
}

listen('overlay:show', (e) => show(e.payload)).catch(() => {})
// initial hide
el.classList.remove('show')