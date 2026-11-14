import { test, expect, audit, deck } from './helpers';

test('authors and reviews an essay, then previews choice import', async ({ app }) => {
  await app.goto('/tests/e2e/harness.html?typed=essay');
  await deck(app);
  await app.getByRole('button', { name: 'إنشاء بطاقة', exact: true }).click();
  await app.getByLabel('نوع السؤال').selectOption('essay');
  await app.getByLabel('السؤال، الصيغة ١', { exact: true }).fill('Explain the orbit.');
  await expect(app.getByLabel('الإجابة، الصيغة ١')).toHaveCount(0);
  await audit(app, 'essay-editor');
  await app.getByRole('button', { name: 'حفظ البطاقة' }).click();
  await app.getByRole('button', { name: 'استيراد بطاقات JSON' }).click();
  await app.getByLabel('محتوى JSON').fill('[{"questionType":"multipleChoice"}]');
  await app.getByRole('button', { name: 'فحص JSON' }).click();
  await app.getByText('عرض الإجابة والملاحظات', { exact: true }).click();
  await expect(app.getByText('إجابة صحيحة', { exact: true })).toBeVisible();
  await audit(app, 'typed-import');
  await app.getByRole('button', { name: 'استيراد البطاقات المحددة' }).click();
  await app.getByRole('button', { name: 'بدء المراجعة', exact: true }).click();
  await expect(app.getByRole('heading', { level: 1, name: 'السؤال' })).toBeFocused();
  const copy = app.getByRole('button', { name: 'نسخ السؤال' });
  await copy.click();
  await expect(copy).toBeFocused();
  await expect(app.getByText('نُسخ السؤال إلى الحافظة.')).toBeVisible();
  await audit(app, 'essay-review');
  await app.getByRole('button', { name: /افتكرتها بسهولة/ }).click();
  await expect(app.getByRole('heading', { name: 'ملخص الجلسة' })).toBeVisible();
});

test('uses native checkbox interaction and presents separate wrong and correct review states', async ({
  app,
}) => {
  await app.goto('/tests/e2e/harness.html?typed=choice');
  await deck(app);
  await app.getByRole('button', { name: 'بدء المراجعة', exact: true }).click();
  const check = app.getByRole('button', { name: 'تحقق من الإجابة' });
  await expect(check).toBeDisabled();
  const rome = app.getByRole('checkbox', { name: 'Rome' });
  await rome.focus();
  await app.keyboard.press('Space');
  await expect(rome).toBeChecked();
  await audit(app, 'choice-options');
  await check.click();
  await expect(app.getByRole('heading', { name: 'النتيجة', level: 2 })).toBeFocused();
  await expect(app.getByRole('group', { name: 'تقييم التذكر' })).toHaveCount(0);
  await audit(app, 'wrong-feedback');
  await app.getByRole('button', { name: 'التراجع عن آخر تقييم' }).click();
  await app.getByRole('checkbox', { name: 'Cairo' }).check();
  await check.click();
  await expect(app.getByRole('heading', { name: 'النتيجة', level: 2 })).toBeFocused();
  await expect(app.getByText('إجابتك صحيحة.')).toBeVisible();
  await audit(app, 'correct-feedback');
  await app.getByRole('button', { name: /افتكرتها بشكل طبيعي/ }).click();
  await expect(app.getByRole('heading', { name: 'ملخص الجلسة' })).toBeVisible();
});
