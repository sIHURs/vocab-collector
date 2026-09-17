import type { VocabularyLogDay } from './types';

export const localDate = (date = new Date()) => `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
export const calendarDate = (value: string) => new Date(`${value}T12:00:00Z`);
export const shiftDate = (value: string, days: number) => {
  const date = calendarDate(value);
  date.setUTCDate(date.getUTCDate() + days);
  return date.toISOString().slice(0, 10);
};
export const logStart = (end: string) => {
  const date = calendarDate(end);
  const month = date.getUTCMonth();
  date.setUTCFullYear(date.getUTCFullYear() - 1);
  if (date.getUTCMonth() !== month) date.setUTCDate(0);
  return shiftDate(date.toISOString().slice(0, 10), 1);
};
export const activityLevel = (count: number) => count === 0 ? 0 : count < 25 ? 1 : count < 50 ? 2 : count < 100 ? 3 : 4;
export const dayLabel = (day: VocabularyLogDay) => {
  const date = calendarDate(day.date).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric', timeZone: 'UTC' });
  if (day.coverage === 'unknown') return `${date} - No activity records are available for this day.`;
  const count = day.count ?? 0;
  const activity = count === 0 ? 'No vocabulary saves recorded.' : `You saved vocabulary ${count === 1 ? 'once' : `${count} times`}.`;
  return `${date} - ${activity}`;
};
