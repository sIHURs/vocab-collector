export const channel = 'vocab-landing-v1';
export type Theme = 'light' | 'dark';
export type HostMessage = { channel: typeof channel; type: 'theme'; theme: Theme };
export type DemoMessage = { channel: typeof channel; type: 'ready' };
export function isThemeMessage(value: unknown): value is HostMessage {
  if (!value || typeof value !== 'object') return false;
  const message = value as Partial<HostMessage>;
  return message.channel === channel && message.type === 'theme' && (message.theme === 'light' || message.theme === 'dark');
}
