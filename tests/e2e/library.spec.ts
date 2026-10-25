import { test, expect, audit, deck } from './helpers';
test('library keyboard, create rename delete and native dialog focus restoration', async ({
  app,
}) => {
  await audit(app, 'library');
  await app.screenshot({ path: 'target/qa-library.png', fullPage: true });
  const create = app.getByRole('button', { name: 'إنشاء مجموعة' });
  await create.focus();
  await app.keyboard.press('Enter');
  await expect(app.getByLabel('اسم المجموعة')).toBeFocused();
  await audit(app, 'create-dialog');
  await app.keyboard.press('Escape');
  await expect(create).toBeFocused();
  await create.click();
  await app.getByLabel('اسم المجموعة').fill('علوم English');
  await app.getByRole('button', { name: 'إنشاء', exact: true }).click();
  await app.getByRole('button', { name: 'إعادة تسمية علوم English' }).click();
  await app.getByLabel('اسم المجموعة').fill('علوم جديدة');
  await app.getByRole('button', { name: 'حفظ الاسم' }).click();
  await app.getByRole('button', { name: 'حذف علوم جديدة' }).click();
  await audit(app, 'delete-dialog');
  await app.getByRole('button', { name: 'حذف المجموعة نهائيًا' }).click();
  await expect(app.getByRole('button', { name: 'علوم جديدة', exact: true })).toHaveCount(0);
});
test('deck collapsed/expanded statistics and mixed text card editor', async ({ app }) => {
  await deck(app);
  await audit(app, 'deck-collapsed');
  await app.getByText('إحصاءات المجموعة', { exact: true }).click();
  await audit(app, 'deck-statistics');
  await app.getByRole('button', { name: 'إنشاء بطاقة', exact: true }).click();
  await audit(app, 'card-editor');
  const fields = app.getByRole('dialog').locator('textarea');
  await fields.nth(0).fill('日本語 <script>literal</script> عربي');
  await fields.nth(1).fill('English إجابة');
  await app.getByRole('button', { name: 'إضافة صيغة بديلة' }).click();
  await audit(app, 'variant-editor');
});
