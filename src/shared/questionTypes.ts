import type { QuestionType } from './generated/QuestionType';

export const questionTypeLabel: Record<QuestionType, string> = {
  recall: 'سؤال وإجابة',
  essay: 'مقالي',
  multipleChoice: 'اختيار من متعدد',
};
