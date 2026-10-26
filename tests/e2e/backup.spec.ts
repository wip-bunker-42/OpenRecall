import { test, expect, audit } from './helpers';
test('backup preview confirmation export and persistent restore notice', async ({ app }) => {
  await app.getByRole('button', { name: 'الإعدادات', exact: true }).click();
  await audit(app, 'settings');
  await app.getByText('النسخ الاحتياطي والاستعادة', { exact: true }).click();
  await app.getByRole('button', { name: 'تصدير جميع البيانات' }).click();
  await expect(app.getByText('اكتمل تصدير النسخة الاحتياطية.')).toBeVisible();
  await audit(app, 'exported-backup');
  await app.getByRole('button', { name: 'اختيار نسخة للاستعادة' }).click();
  await audit(app, 'restore-preview');
  await app.getByLabel('أوافق على استبدال جميع بيانات المكتبة الحالية').check();
  await app.getByRole('button', { name: 'استعادة النسخة واستبدال المكتبة' }).click();
  await expect(app.getByText('تمت استعادة النسخة الاحتياطية.')).toBeVisible();
  await expect(
    app.getByRole('button', { name: 'إظهار نسخة الأمان السابقة للاستعادة' }),
  ).toBeVisible();
  await audit(app, 'restore-completed');
});
