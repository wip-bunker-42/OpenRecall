export const number = (v: number) => new Intl.NumberFormat('ar-EG').format(v);
export const dateTime = (s: string, timezone = 'Africa/Cairo') =>
  new Intl.DateTimeFormat('ar-EG', {
    dateStyle: 'medium',
    timeStyle: 'short',
    timeZone: timezone,
  }).format(new Date(s));
export function interval(seconds: number): string {
  const unit = seconds < 60 ? 'ثانية' : seconds < 3600 ? 'دقيقة' : seconds < 86400 ? 'ساعة' : 'يوم';
  const divisor = seconds < 60 ? 1 : seconds < 3600 ? 60 : seconds < 86400 ? 3600 : 86400;
  return `بعد نحو ${number(Math.max(1, Math.round(seconds / divisor)))} ${unit}`;
}
