import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@fontsource-variable/noto-sans-arabic';
import './styles/global.css';
import { App } from './app/App';
import { createNativeApi, createTauriApi } from './shared/api';
const api = createTauriApi();
const native = createNativeApi();
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App api={api} native={native} />
  </StrictMode>,
);
