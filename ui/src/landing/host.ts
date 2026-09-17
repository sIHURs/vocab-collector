import { channel, type HostMessage, type Theme } from './protocol';
import './host.css';
const frame = document.querySelector<HTMLIFrameElement>('#demo')!;
const button = document.querySelector<HTMLButtonElement>('#theme')!;
let theme: Theme = 'light';
const sendTheme = () => frame.contentWindow?.postMessage({ channel, type: 'theme', theme } satisfies HostMessage, location.origin);
button.addEventListener('click', () => {
  theme = theme === 'light' ? 'dark' : 'light';
  document.documentElement.dataset.appearance = theme;
  button.textContent = `Switch to ${theme === 'light' ? 'dark' : 'light'}`;
  sendTheme();
});
window.addEventListener('message', event => {
  if (event.origin !== location.origin || event.source !== frame.contentWindow) return;
  if (event.data?.channel !== channel || event.data?.type !== 'ready') return;
  document.querySelector('#status')!.textContent = 'Demo ready. Capture a word, explore Vocabulary, then try Review.';
  sendTheme();
});
