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
export const activityLevel = (count: number) => count === 0 ? 0 : count < 3 ? 1 : count < 6 ? 2 : count < 10 ? 3 : 4;
export const dayLabel = (day: VocabularyLogDay) => `${day.date}: ${day.coverage === 'unknown' ? 'history unknown' : `${day.count ?? 0} capture${day.count === 1 ? '' : 's'}${day.coverage === 'partial' ? ' recorded; partial history' : ''}`}`;
