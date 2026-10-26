import { test, expect, audit, deck } from './helpers';
test('import preview exposes invalid and duplicate forms and requires their exclusion', async ({
  app,
}) => {
  await deck(app);
  await app.getByRole('button', { name: 'استيراد بطاقات JSON' }).click();
  await audit(app, 'import-input');
  await app.locator('dialog textarea').fill('[]');
  await app.getByRole('button', { name: 'فحص JSON' }).click();
  await expect(app.getByRole('heading', { name: 'معاينة البطاقات' })).toBeFocused();
  await audit(app, 'import-invalid-duplicate');
  const commit = app.getByRole('button', { name: 'استيراد البطاقات المحددة' });
  await expect(commit).toBeDisabled();
  await app.getByRole('button', { name: /استبعاد.*غير الصالحة/ }).click();
  await expect(commit).toBeEnabled();
  await commit.click();
  await expect(app.getByRole('dialog')).toHaveCount(0);
});
