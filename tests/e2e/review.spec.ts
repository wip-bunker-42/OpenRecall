import { test, expect, audit } from './helpers';
test('review headings, plaintext bidi, interval accessible names, filters, summary undo and keyboard', async ({
  app,
}) => {
  await app.getByRole('button', { name: 'بدء المراجعة، طب' }).click();
  await expect(app.getByRole('heading', { level: 1, name: 'السؤال', exact: true })).toBeFocused();
  await expect(app.getByText('<h1>مرحبا</h1>', { exact: true })).toHaveAttribute('dir', 'auto');
  await audit(app, 'question');
  await app.keyboard.press('Space');
  await expect(app.getByRole('heading', { level: 2, name: 'الإجابة', exact: true })).toBeFocused();
  await expect(
    app.getByRole('heading', { level: 3, name: 'الملاحظات', exact: true }),
  ).toBeVisible();
  await expect(app.getByRole('button', { name: /افتكرتها بسهولة.*بعد نحو.*٣٠/ })).toBeVisible();
  await audit(app, 'revealed-answer');
  await app.screenshot({ path: 'target/qa-review.png', fullPage: true });
  await app.getByLabel('بطاقات سبق تقييمها').check();
  await expect(app.getByRole('heading', { name: 'لا توجد بطاقات مطابقة الآن' })).toBeVisible();
  await audit(app, 'empty-filter');
  await app.getByLabel('البطاقات الجديدة فقط').check();
  await expect(app.getByRole('heading', { name: 'الإجابة', exact: true })).toBeVisible();
  await app.getByRole('button', { name: /افتكرتها بسهولة/ }).click();
  await expect(app.getByRole('heading', { name: 'ملخص الجلسة' })).toBeFocused();
  await audit(app, 'summary');
  await app.keyboard.press('Control+z');
  await expect(app.getByRole('heading', { name: 'الإجابة', exact: true })).toBeFocused();
});
