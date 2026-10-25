import { test, expect, audit } from './helpers';
test('profile advanced fields and insufficient-data gate remain accessible', async ({ app }) => {
  await app.getByRole('button', { name: 'نماذج التحسين', exact: true }).click();
  await audit(app, 'profiles-collapsed');
  await app.getByText('نموذج مشترك', { exact: true }).click();
  await audit(app, 'profile-expanded');
  await app.getByText('إعدادات التدريب المتقدمة', { exact: true }).click();
  await expect(app.getByLabel(/Schedule Penalties/)).toBeChecked();
  await audit(app, 'profile-advanced');
  await app.getByText('تحسين النموذج', { exact: true }).click();
  await expect(app.getByRole('button', { name: 'بدء التحسين' })).toBeDisabled();
  await audit(app, 'training-insufficient');
});
