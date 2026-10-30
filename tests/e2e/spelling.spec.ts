import { test, expect, audit } from './helpers';
async function create(app: import('@playwright/test').Page) {
  await app.getByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' }).click();
  await expect(app.getByLabel('المجموعة المصدر')).toBeFocused();
  await audit(app, 'spelling-setup');
  await app.getByLabel('المجموعة المصدر').selectOption('deck-1');
  await app.getByRole('button', { name: 'إنشاء وربط المصدر' }).click();
  await expect(
    app.getByRole('heading', { level: 1, name: 'مجموعة التدرب على الكتابة' }),
  ).toBeVisible();
}
test('spelling focus, disposable input, saved context, direct grade, filter and undo accessibility', async ({
  app,
}) => {
  await create(app);
  await expect(app.getByRole('button', { name: 'استيراد بطاقات JSON' })).toHaveCount(0);
  await audit(app, 'spelling-deck');
  await app.getByRole('button', { name: 'بدء المراجعة', exact: true }).click();
  const field = app.getByRole('textbox', { name: 'مساحة الكتابة' });
  await expect(field).toHaveValue('English\n');
  await expect(field).toBeFocused();
  expect(
    await field.evaluate((e: HTMLTextAreaElement) => [e.selectionStart, e.selectionEnd]),
  ).toEqual([0, 0]);
  await expect(app.getByRole('heading', { name: 'الكلمة', level: 1 })).toBeVisible();
  await expect(app.getByRole('button', { name: 'عرض الإجابة' })).toHaveCount(0);
  await audit(app, 'spelling-review');
  const marker = 'browser-private-attempt-marker';
  await field.fill(marker);
  await app.getByText('عرض البطاقة الأصلية', { exact: true }).click();
  await expect(app.getByText('معنى محفوظ', { exact: true })).toBeVisible();
  await expect(field).toHaveValue(marker);
  await audit(app, 'spelling-context');
  await app.getByLabel('البطاقات الجديدة فقط').check();
  await expect(field).toHaveValue(marker);
  await app.getByLabel('بطاقات سبق تقييمها').check();
  await expect(field).toHaveCount(0);
  await audit(app, 'spelling-parked');
  await app.getByLabel('كل البطاقات المستحقة').check();
  await expect(field).toHaveValue('English\n');
  await field.fill(marker);
  await app.getByRole('button', { name: /افتكرتها بسهولة/ }).click();
  await expect(app.getByRole('heading', { name: 'ملخص الجلسة' })).toBeVisible();
  await app.getByRole('button', { name: 'التراجع عن آخر تقييم' }).click();
  await expect(field).toHaveValue('English\n');
  await expect(field).toBeFocused();
  expect(
    await app.evaluate(() =>
      JSON.stringify((window as unknown as { presentationCalls: unknown[] }).presentationCalls),
    ),
  ).not.toContain(marker);
  await audit(app, 'spelling-undo');
  await app.getByRole('button', { name: /افتكرتها بسهولة/ }).click();
  await app.getByRole('button', { name: 'المجموعات', exact: true }).click();
  await expect(app.getByRole('button', { name: 'فتح مجموعة التدرب على الكتابة' })).toBeVisible();
  await expect(app.getByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' })).toHaveCount(0);
  await app.getByText('إحصاءات المكتبة', { exact: true }).click();
  await expect(app.getByText('منها كلمات الإملاء', { exact: true })).toBeVisible();
  await audit(app, 'spelling-global-completion');
});
test('generated word exclusion and source deletion remain accessible', async ({ app }) => {
  await create(app);
  await app.getByText('البطاقات', { exact: true }).click();
  await app.getByRole('button', { name: 'حذف البطاقة ١', exact: true }).click();
  await audit(app, 'spelling-word-delete');
  await app.getByRole('button', { name: 'حذف البطاقة نهائيًا' }).click();
  await app.getByText('مصدر كلمات الكتابة', { exact: true }).click();
  await expect(app.getByLabel('المجموعة المصدر')).toHaveValue('deck-1');
  await app.getByText('الكلمات المستبعدة', { exact: true }).click();
  await expect(app.getByRole('button', { name: 'السماح بجمع english مجددًا' })).toBeVisible();
  await audit(app, 'spelling-exclusions');
  await app.getByRole('button', { name: 'السماح بجمع english مجددًا' }).click();
  await expect(app.getByText('لا توجد كلمات مستبعدة.')).toBeVisible();
  await app.getByRole('button', { name: 'العودة إلى المجموعات' }).click();
  await app.getByRole('button', { name: 'حذف طب', exact: true }).click();
  await app.getByRole('button', { name: 'حذف المجموعة نهائيًا' }).click();
  await app.getByRole('button', { name: 'فتح مجموعة التدرب على الكتابة' }).click();
  await app.getByText('مصدر كلمات الكتابة', { exact: true }).click();
  await expect(app.getByText(/حُذفت المجموعة المصدر/)).toBeVisible();
  await audit(app, 'spelling-source-deleted');
});
