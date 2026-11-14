import { $, browser, expect, click, heading, audit, root, command, containing } from './helpers';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const deckName = 'اختبار أنواع الأسئلة';
const statePath = resolve(root, 'question-types-state.json');
const resume = process.env.OPENRECALL_NATIVE_QUESTION_PHASE === 'resume';

describe('native typed questions and durable feedback', () => {
  if (resume) {
    it('restores wrong feedback and its undo after a real process restart', async () => {
      const saved = JSON.parse(readFileSync(statePath, 'utf8'));
      await heading('مجموعاتي');
      await click(`استكمال مراجعة ${deckName}`);
      await heading('اختر مدينة');
      await expect($('h2=النتيجة')).toBeFocused();
      const view = await command('readSession', { sessionId: saved.sessionId });
      expect(view.state).toBe('choiceFeedback');
      if (view.state !== 'choiceFeedback') throw new Error('Native feedback was not retained');
      expect(view.payload.feedbackId).toBe(saved.feedbackId);
      const gradedCard = await command('getCard', { cardId: view.payload.cardId });
      expect(gradedCard.reviewCount).toBe(1);
      await audit('native-choice-feedback-restart');
      const arrival = await command('createCard', {
        deckId: gradedCard.deckId,
        draft: {
          forms: [
            {
              id: null,
              front: 'Newly due during correction.',
              back: 'Arrived.',
              notes: '',
              language: 'en',
              questionTitle: 'وصلت بطاقة',
              questionType: 'recall',
              multipleChoice: null,
            },
          ],
        },
      });
      await $('input[value="new"]').click();
      await expect($('input[value="new"]')).toBeChecked();
      await click('متابعة المراجعة');
      await heading('وصلت بطاقة');
      await click('التراجع عن آخر تقييم');
      await expect($('.choice-options input:checked')).toExist();
      expect((await command('getCard', { cardId: view.payload.cardId })).reviewCount).toBe(0);
      expect((await command('getCard', { cardId: arrival.id })).bag).toEqual(arrival.bag);
      await (await containing('.choice-options label', 'Cairo')).click();
      await click('تحقق من الإجابة');
      await expect($('h2=النتيجة')).toBeFocused();
      await audit('native-choice-correct');
      await $('.rating-easy').click();
      await heading('وصلت بطاقة');
      await click('عرض الإجابة');
      await $('.rating-easy').click();
      await heading('ملخص الجلسة');
      expect((await command('getCard', { cardId: view.payload.cardId })).reviewCount).toBe(1);
    });
  } else {
    it('copies and rates an essay, imports choices and commits a wrong answer only once', async () => {
      await heading('مجموعاتي');
      await click('إنشاء مجموعة');
      await $('dialog input').setValue(deckName);
      await click('إنشاء');
      await click(deckName);
      const deck = (await command('listDecks', {})).find((item) => item.name === deckName)!;
      await command('createCard', {
        deckId: deck.id,
        draft: {
          forms: [
            {
              id: null,
              front: 'Explain an orbit.',
              back: '',
              notes: 'Private synthetic context.',
              language: 'en',
              questionTitle: 'اشرح',
              questionType: 'essay',
              multipleChoice: null,
            },
          ],
        },
      });
      await click('بدء المراجعة');
      await heading('اشرح');
      await click('نسخ السؤال');
      await expect($('p=نُسخ السؤال إلى الحافظة.')).toBeDisplayed();
      await expect($('button=نسخ السؤال')).toBeFocused();
      await audit('native-essay-copy');
      await $('.rating-easy').click();
      await heading('ملخص الجلسة');
      await click('العودة إلى المجموعة');
      writeFileSync(
        resolve(root, 'cards.json'),
        JSON.stringify([
          {
            questionType: 'multipleChoice',
            questionTitle: 'اختر مدينة',
            front: 'Which is in Egypt?',
            notes: 'Cairo is in Egypt.',
            language: 'en',
            shuffleChoices: false,
            choices: [
              { key: 'a', text: 'Cairo' },
              { key: 'b', text: 'Rome' },
            ],
            correctChoices: ['a'],
          },
        ]),
      );
      await click('استيراد بطاقات JSON');
      await click('اختيار ملف JSON');
      await click('فحص JSON');
      await expect($('h3=معاينة البطاقات')).toBeDisplayed();
      await click('استيراد البطاقات المحددة');
      await expect($('dialog')).not.toExist();
      await click('بدء المراجعة');
      await heading('اختر مدينة');
      await audit('native-choice-question');
      const session = (await command('bootstrap', {})).resumableSessions.find(
        (item) => item.deckId === deck.id,
      )!;
      const frame = await command('readSession', { sessionId: session.sessionId });
      if (frame.state !== 'question') throw new Error('Expected a reserved choice question');
      await command('rateCard', {
        sessionId: session.sessionId,
        presentationId: frame.payload.presentationId,
        expectedCardRevision: frame.payload.cardRevision,
        actionId: crypto.randomUUID(),
        rating: 'easy',
      }).then(
        () => {
          throw new Error('Native grading accepted an unconfirmed choice');
        },
        (error: unknown) => expect(String(error)).toContain('answerHidden'),
      );
      await (await containing('.choice-options label', 'Rome')).click();
      await click('تحقق من الإجابة');
      await expect($('h2=النتيجة')).toBeFocused();
      await expect($('.rating-fieldset')).not.toExist();
      const view = await command('readSession', { sessionId: session.sessionId });
      if (view.state !== 'choiceFeedback') throw new Error('Expected committed wrong feedback');
      expect(view.payload.lastReview?.rating).toBe('again');
      expect((await command('getCard', { cardId: frame.payload.card.id })).reviewCount).toBe(1);
      await click('المجموعات');
      await click(`استكمال مراجعة ${deckName}`);
      await expect($('h2=النتيجة')).toBeFocused();
      await audit('native-choice-feedback');
      writeFileSync(
        statePath,
        JSON.stringify({ sessionId: session.sessionId, feedbackId: view.payload.feedbackId }),
      );
      await browser.execute(() => window.dispatchEvent(new Event('blur')));
    });
  }
});
