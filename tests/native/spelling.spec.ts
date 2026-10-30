import { $, browser, expect, click, heading, audit, root, containing, command } from './helpers';
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { ExportReport } from '../../src/shared/generated/ExportReport';
const marker = 'native-private-spelling-attempt-73bc40';
const restartMarker = 'native-restart-disposable-attempt-93c820';
const statePath = resolve(root, 'spelling-state.json');
async function selectSource(selector: string, id: string) {
  await $(selector).waitForEnabled();
  await browser.execute(
    (selector, id) => {
      const field = document.querySelector<HTMLSelectElement>(selector)!;
      field.value = id;
      field.dispatchEvent(new Event('change', { bubbles: true }));
    },
    selector,
    id,
  );
}
async function importCards(cards: unknown) {
  writeFileSync(resolve(root, 'cards.json'), JSON.stringify(cards));
  await click('استيراد بطاقات JSON');
  await click('اختيار ملف JSON');
  await click('فحص JSON');
  await expect($('h3=معاينة البطاقات')).toBeDisplayed();
  await click('استيراد البطاقات المحددة');
  await expect($('dialog')).not.toExist();
}
async function restore(path: string) {
  copyFileSync(path, resolve(root, 'restore.json'));
  await click('الإعدادات');
  await $('summary=النسخ الاحتياطي والاستعادة').click();
  await click('اختيار نسخة للاستعادة');
  await expect($('h2=معاينة الاستعادة')).toBeDisplayed();
  await $('dialog input[type="checkbox"]').click();
  await click('استعادة النسخة واستبدال المكتبة');
  await heading('مجموعاتي');
}
async function exportInPlace(): Promise<ExportReport> {
  const result = (await browser.executeAsync<unknown, []>((done) => {
    const native = window as unknown as {
      __TAURI_INTERNALS__: { invoke: (name: string) => Promise<ExportReport> };
    };
    native.__TAURI_INTERNALS__
      .invoke('export_backup')
      .then(done)
      .catch((error) => done({ error }));
  })) as ExportReport | { error: unknown };
  if ('error' in result) throw new Error(JSON.stringify(result.error));
  return result;
}
function assertNoAttempt(path: string) {
  const text = readFileSync(path, 'utf8');
  expect(text).not.toContain(marker);
  expect(text).not.toContain(restartMarker);
  for (const name of ['openrecall.sqlite3', 'openrecall.sqlite3-wal']) {
    const path = resolve(root, 'data', name);
    if (existsSync(path))
      for (const value of [marker, restartMarker])
        expect(readFileSync(path).includes(Buffer.from(value))).toBe(false);
  }
}
async function gradeEasy() {
  const previous = await $('.spelling-word').getText();
  await $('.rating-easy').waitForEnabled();
  await $('.rating-easy').click();
  await browser.waitUntil(
    async () =>
      (await $('h1=ملخص الجلسة').isExisting()) ||
      (await $('.spelling-word').getText()) !== previous,
  );
}
describe('native spelling privacy, portability and process restart', () => {
  if (process.env.OPENRECALL_NATIVE_SPELLING_PHASE === 'resume') {
    it('resumes the actual saved presentation with only its target and no previous attempt', async () => {
      const saved = JSON.parse(readFileSync(statePath, 'utf8'));
      await heading('مجموعاتي');
      await click('استكمال مراجعة مجموعة التدرب على الكتابة');
      await heading('الكلمة');
      await expect($('textarea')).toHaveValue(`${saved.word}\n`, { trim: false });
      await expect($('textarea')).toBeFocused();
      expect(
        await browser.execute(() => {
          const t = document.querySelector('textarea')!;
          return [t.selectionStart, t.selectionEnd];
        }),
      ).toEqual([0, 0]);
      const frame = await command('readSession', { sessionId: saved.sessionId });
      if (frame.state !== 'question') throw new Error('Saved question did not resume');
      expect(frame.payload.presentationId).toBe(saved.presentationId);
      expect(frame.payload.card.id).toBe(saved.cardId);
      expect(frame.payload.card.reviewCount).toBe(0);
      expect(frame.payload.card.memory).toBeNull();
      await $('summary=عرض البطاقة الأصلية').click();
      await expect($('h3=الإجابة')).toBeDisplayed();
      await audit('native-spelling-restarted-target-only');
      await gradeEasy();
      await heading('ملخص الجلسة');
      const stats = await command('getLibraryStatistics', {});
      expect(stats.spellingCompletedToday).toBe(6);
      const report = await exportInPlace();
      assertNoAttempt(report.path);
      writeFileSync(
        resolve(root, 'spelling-resume-result.json'),
        JSON.stringify({
          presentationPreserved: true,
          attemptDiscarded: true,
          completedWords: stats.spellingCompletedToday,
          backup: report.path,
        }),
      );
    });
  } else {
    it('collects, preserves sources, restores both formats and prepares a disposable restart attempt', async function () {
      this.timeout(300_000);
      await heading('مجموعاتي');
      await click('إنشاء مجموعة');
      await $('dialog input').setValue('مصدر الإملاء');
      await click('إنشاء');
      await click('مصدر الإملاء');
      await importCards([
        {
          questionTitle: 'ترجم',
          front: 'red blue',
          back: 'أحمر وأزرق',
          notes: 'green',
          variants: [
            { questionTitle: 'أكمل', front: 'blue yellow', back: 'أزرق وأصفر', notes: 'red' },
          ],
        },
      ]);
      await click('بدء المراجعة');
      expect(['ترجم', 'أكمل']).toContain(await $('h1').getText());
      await audit('native-custom-question-title');
      await click('إنهاء الجلسة');
      await heading('ملخص الجلسة');
      await click('العودة إلى المجموعة');
      await importCards([{ front: 'white', back: 'أبيض', notes: '' }]);
      await click('المجموعات');
      const source = (await command('listDecks', {})).find((d) => d.name === 'مصدر الإملاء')!;
      await click('إنشاء مجموعة التدرب على الكتابة');
      await selectSource('dialog select', source.id);
      await audit('native-spelling-setup');
      await click('إنشاء وربط المصدر');
      await heading('مجموعة التدرب على الكتابة');
      const spelling = (await command('listDecks', {})).find((d) => d.kind === 'spelling')!;
      expect(spelling.logicalCards).toBe(5);
      expect(await $('button=استيراد بطاقات JSON').isExisting()).toBe(false);
      await click('بدء المراجعة');
      await heading('الكلمة');
      const word = await $('.spelling-word').getText();
      await expect($('textarea')).toHaveValue(`${word}\n`, { trim: false });
      await expect($('textarea')).toBeFocused();
      expect(
        await browser.execute(() => {
          const t = document.querySelector('textarea')!;
          return [t.selectionStart, t.selectionEnd];
        }),
      ).toEqual([0, 0]);
      await expect($('.rating-easy')).not.toHaveAttribute('aria-describedby');
      await audit('native-spelling-initial');
      await $('textarea').setValue(marker);
      const bootstrap = await command('bootstrap', {});
      const session = bootstrap.resumableSessions.find((s) => s.deckId === spelling.id)!;
      const held = await command('readSession', { sessionId: session.sessionId });
      if (held.state !== 'question') throw new Error('No reserved word');
      const origins = await command('listSpellingSources', {
        cardId: held.payload.card.id,
        page: 1,
        pageSize: 20,
      });
      const sourceCard = await command('getCard', {
        cardId: origins.items[0].snapshot.sourceCardId,
      });
      await command('updateCard', {
        cardId: sourceCard.id,
        expectedRevision: sourceCard.revision,
        draft: {
          forms: sourceCard.forms.map((f) => ({
            ...f,
            notes: f.id === origins.items[0].snapshot.sourceFormId ? `${f.notes} orange` : f.notes,
          })),
        },
      });
      expect((await command('getDeck', { deckId: spelling.id })).logicalCards).toBe(6);
      await browser.execute(() => window.dispatchEvent(new Event('focus')));
      await expect($('.spelling-word')).toHaveText(word);
      await expect($('textarea')).toHaveValue(marker);
      const freshSource = await command('getDeck', { deckId: source.id });
      await command('deleteDeck', { deckId: source.id, expectedRevision: freshSource.revision });
      await $('summary=عرض البطاقة الأصلية').click();
      await expect($('p*=حُذفت البطاقة الأصلية')).toBeDisplayed();
      await expect($('textarea')).toHaveValue(marker);
      await audit('native-spelling-retained-deleted-source');
      await gradeEasy();
      expect(await $('textarea').getValue()).not.toContain(marker);
      for (let i = 0; i < 10 && !(await $('h1=ملخص الجلسة').isExisting()); i++) await gradeEasy();
      await heading('ملخص الجلسة');
      expect((await command('getLibraryStatistics', {})).spellingCompletedToday).toBe(6);
      await click('التراجع عن آخر تقييم');
      const undone = await $('.spelling-word').getText();
      await expect($('textarea')).toHaveValue(`${undone}\n`, { trim: false });
      await expect($('textarea')).toBeFocused();
      await audit('native-spelling-summary-undo');
      await gradeEasy();
      await heading('ملخص الجلسة');
      await click('العودة إلى المجموعة');
      await $('summary=مصدر كلمات الكتابة').click();
      await expect($('p*=حُذفت المجموعة المصدر')).toBeDisplayed();
      await click('الإعدادات');
      await $('summary=النسخ الاحتياطي والاستعادة').click();
      await click('تصدير جميع البيانات');
      await expect($('p=اكتمل تصدير النسخة الاحتياطية.')).toBeDisplayed();
      const portable = await $('.result p[dir="auto"]').getText();
      const backup = JSON.parse(readFileSync(portable, 'utf8'));
      expect(backup.formatVersion).toBe(2);
      expect(backup.data.spelling.words).toHaveLength(6);
      assertNoAttempt(portable);
      await click('المجموعات');
      await $('button[aria-label="حذف مجموعة التدرب على الكتابة"]').click();
      await click('حذف المجموعة نهائيًا');
      await expect($('button=فتح مجموعة التدرب على الكتابة')).not.toExist();
      await restore(portable);
      expect((await command('getDeck', { deckId: spelling.id })).logicalCards).toBe(6);
      const legacyPath = resolve('crates/openrecall-core/tests/fixtures/backups/v0.1.0.json');
      const legacy = JSON.parse(readFileSync(legacyPath, 'utf8'));
      await restore(legacyPath);
      await expect($(`button=${legacy.data.decks[0].name}`)).toBeDisplayed();
      await audit('native-genuine-legacy-restored');
      await click(`استكمال مراجعة ${legacy.data.decks[0].name}`);
      await click('التراجع عن آخر تقييم');
      await expect($('h2=الإجابة')).toBeDisplayed();
      await audit('native-genuine-legacy-undo');
      await click('إنهاء الجلسة');
      await heading('ملخص الجلسة');
      await click('المجموعات');
      await restore(portable);
      expect((await command('getLibraryStatistics', {})).spellingCompletedToday).toBe(6);
      const words = await command('listCards', {
        deckId: spelling.id,
        query: '',
        page: 1,
        pageSize: 20,
      });
      const target = words.items[0];
      const targetWord = target.forms[0].front;
      await click('إنشاء مجموعة');
      await $('dialog input').setValue('مصدر ثان للإملاء');
      await click('إنشاء');
      await click('مصدر ثان للإملاء');
      await importCards([{ front: targetWord, back: 'كلمة المصدر الثاني', notes: '' }]);
      await click('المجموعات');
      const second = (await command('listDecks', {})).find((d) => d.name === 'مصدر ثان للإملاء')!;
      await click('فتح مجموعة التدرب على الكتابة');
      await $('summary=مصدر كلمات الكتابة').click();
      await $('select').waitForExist();
      await $('select').waitForEnabled();
      await $('summary=الجدولة والتحسين').click();
      const policy = $('input[name="minIntervalSeconds"]');
      await policy.setValue('120');
      await expect(policy).toHaveValue('120');
      await expect($('select')).toBeDisabled();
      await expect($('button=فحص المصدر مجددًا')).toBeDisabled();
      await audit('native-spelling-policy-draft-guard');
      await click('تجاهل تعديلات المجموعة');
      await expect($('select')).toBeEnabled();
      const beforeDraft = await command('bootstrap', {});
      await selectSource('select', second.id);
      await expect($('button=بدء المراجعة')).toBeDisabled();
      await expect(policy).toBeDisabled();
      await click('المجموعات');
      await expect($('h2=تغييرات غير محفوظة')).toBeDisplayed();
      await click('متابعة التعديل');
      await expect($('select')).toHaveValue(second.id);
      await expect($('button=بدء المراجعة')).toBeDisabled();
      const afterCancel = await command('bootstrap', {});
      expect(afterCancel.resumableSessions).toEqual(beforeDraft.resumableSessions);
      expect(afterCancel.libraryRevision).toBe(beforeDraft.libraryRevision);
      await audit('native-spelling-source-draft-guard');
      await click('حفظ المصدر');
      await expect($('p*=أُضيفت ٠ كلمة')).toBeDisplayed();
      await $('summary=البطاقات').click();
      const row = await containing('li.collection', targetWord);
      await row.$('button.danger').click();
      await click('حذف البطاقة نهائيًا');
      expect((await command('getDeck', { deckId: spelling.id })).logicalCards).toBe(5);
      await click('فحص المصدر مجددًا');
      expect((await command('getDeck', { deckId: spelling.id })).logicalCards).toBe(5);
      await $('summary=الكلمات المستبعدة').click();
      const allow = await $(`button=السماح بجمع ${targetWord.toLowerCase()} مجددًا`);
      await allow.waitForExist();
      await audit('native-spelling-excluded');
      await allow.click();
      await expect($('p=لا توجد كلمات مستبعدة.')).toBeDisplayed();
      const recollected = (
        await command('listCards', {
          deckId: spelling.id,
          query: targetWord,
          page: 1,
          pageSize: 20,
        })
      ).items.find((c) => c.forms[0].front === targetWord)!;
      expect(recollected.id).not.toBe(target.id);
      expect(recollected.reviewCount).toBe(0);
      expect(recollected.memory).toBeNull();
      await click('بدء المراجعة');
      await heading('الكلمة');
      await expect($('.spelling-word')).toHaveText(targetWord);
      await $('textarea').setValue(restartMarker);
      const resumable = (await command('bootstrap', {})).resumableSessions.find(
        (s) => s.deckId === spelling.id,
      )!;
      const pending = await command('readSession', { sessionId: resumable.sessionId });
      if (pending.state !== 'question') throw new Error('No restart presentation');
      const report = await exportInPlace();
      assertNoAttempt(report.path);
      writeFileSync(
        statePath,
        JSON.stringify({
          word: targetWord,
          cardId: recollected.id,
          presentationId: pending.payload.presentationId,
          sessionId: resumable.sessionId,
          backup: report.path,
        }),
      );
      await expect($('textarea')).toHaveValue(restartMarker);
    });
  }
});
