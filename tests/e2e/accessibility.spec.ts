import { test, expect, audit } from './helpers';
test('settings validation error, readonly information and updater explicit state', async ({
  app,
}) => {
  await app.getByRole('button', { name: 'الإعدادات', exact: true }).click();
  await app.getByText('معلومات التطبيق', { exact: true }).click();
  await audit(app, 'system-information');
  await app.getByText('تحديث التطبيق', { exact: true }).click();
  await audit(app, 'updater-idle');
  await app.getByRole('button', { name: /التحقق من.*تحديث/ }).click();
  await expect(app.getByText(/تستخدم أحدث/)).toBeVisible();
  await audit(app, 'updater-current');
  await app.getByLabel('الاحتفاظ المطلوب (٪)', { exact: true }).fill('101');
  await app.getByRole('button', { name: 'حفظ الإعدادات', exact: true }).click();
  await expect(app.getByRole('alert')).toBeVisible();
  await audit(app, 'settings-error');
});
