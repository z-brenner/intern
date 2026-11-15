import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserApp } from './BrowserApp';
import { takeOnlyAimedPresses } from './lib/aimedPress';
import './styles/base.css';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Intern could not find its root element.');
}

// Before anything is drawn, so every control that acts is seen as it appears.
takeOnlyAimedPresses();

createRoot(root).render(
  <StrictMode>
    <BrowserApp />
  </StrictMode>,
);
