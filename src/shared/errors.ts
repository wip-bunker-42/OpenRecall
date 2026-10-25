export function errorText(error: unknown): string {
  const code =
    typeof error === 'object' && error !== null && 'code' in error ? String(error.code) : 'unknown';
  const messages: Record<string, string> = {
    storage: 'تعذر حفظ البيانات. أعد المحاولة وتأكد من توفر مساحة على الجهاز.',
    fileAccess: 'تعذر فتح الملف أو حفظه. تحقق من صلاحية الوصول إلى المجلد.',
    validation: 'راجع الحقول المطلوبة والقيم المسموح بها.',
    conflict: 'تغيرت البيانات منذ فتح هذه الصفحة. حدّث الصفحة وأعد المحاولة.',
    reviewInProgress: 'أكمل جلسة المراجعة أو أنهها أولًا قبل تغيير نموذج الجدولة.',
    insufficientData: 'البيانات غير كافية للتدريب. راجع شروط التدريب أولًا.',
    notFound: 'العنصر غير موجود. حدّث الصفحة.',
    responseMismatch: 'تعذر قراءة استجابة التطبيق. أغلق الصفحة وافتحها مجددًا.',
  };
  return messages[code] ?? `تعذر إكمال العملية (${code}). أعد المحاولة.`;
}
