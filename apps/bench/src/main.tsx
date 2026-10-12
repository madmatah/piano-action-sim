/**
 * Single-key bench.
 *
 * This is the application that has to be right. It is where a technician
 * regulates, measures and trains, so every number it shows is a claim about
 * physics. Its first milestone is deliberately not an animation: it is a
 * weighing bench, because the weighing comes out of a static solve with no time
 * integration at all, so no integrator bug can mask a geometry error.
 */
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './App'
import './style.css'

const root = document.getElementById('root')
if (!root) throw new Error('no #root in the document')
createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
