export function validation(field: string, code = 'range'): never {
  throw { code: 'validation', fields: [{ field, code }], context: null };
}
export function numeric(
  data: FormData,
  name: string,
  min: number,
  max: number,
  integer?: boolean,
  optional?: false,
): number;
export function numeric(
  data: FormData,
  name: string,
  min: number,
  max: number,
  integer: boolean,
  optional: true,
): number | null;
export function numeric(
  data: FormData,
  name: string,
  min: number,
  max: number,
  integer = false,
  optional = false,
): number | null {
  const raw = String(data.get(name) ?? '').trim();
  if (optional && !raw) return null;
  const value = Number(raw);
  if (
    !raw ||
    !Number.isFinite(value) ||
    value < min ||
    value > max ||
    (integer && !Number.isInteger(value))
  )
    return validation(name);
  return value;
}
export function fieldError(error: unknown, name: string): string | null {
  if (typeof error !== 'object' || !error || !('fields' in error) || !Array.isArray(error.fields))
    return null;
  const issue = error.fields.find((f) => f?.field === name);
  if (!issue) return null;
  return issue.code === 'invalidTimezone'
    ? 'أدخل منطقة زمنية صحيحة مثل Africa/Cairo.'
    : issue.code === 'requiredOrTooLong'
      ? 'أدخل اسمًا من ١ إلى ٢٥٦ حرفًا.'
      : 'راجع القيمة والمدى المسموح به في هذا الحقل.';
}
