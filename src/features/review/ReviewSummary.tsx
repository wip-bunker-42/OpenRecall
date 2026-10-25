import { useEffect, useRef } from 'react';
import type { SessionSummary } from '../../shared/generated/SessionSummary';
import { number, dateTime } from '../../shared/format';
import { ratings, ratingLabels } from './RatingButtons';
export function ReviewSummary({
  summary,
  timezone,
}: {
  summary: SessionSummary;
  timezone: string;
}) {
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    heading.current?.focus();
  }, []);
  return (
    <>
      <h1 tabIndex={-1} ref={heading}>
        ملخص الجلسة
      </h1>
      <p>
        {summary.allDueRemaining > 0
          ? 'انتهت الجلسة. ما زالت هناك بطاقات مستحقة يمكنك مراجعتها في جلسة أخرى.'
          : 'اكتملت المراجعات المتاحة في هذه الجلسة.'}{' '}
        تقدمك محفوظ على جهازك.
      </p>
      <dl className="statistics-list">
        <dt>محاولات التذكر</dt>
        <dd>{number(summary.attempts)}</dd>
        <dt>بطاقات مختلفة</dt>
        <dd>{number(summary.uniqueCards)}</dd>
        <dt>بطاقات جديدة</dt>
        <dd>{number(summary.newCards)}</dd>
        <dt>وقت المذاكرة الفعلي</dt>
        <dd>{number(Math.round(summary.activeMs / 1000))} ثانية</dd>
        <dt>مدة الجلسة الكلية</dt>
        <dd>{number(Math.round(summary.wallMs / 1000))} ثانية</dd>
        <dt>متوسط وقت الإجابة</dt>
        <dd>
          {summary.meanResponseMs === null
            ? 'لا توجد تقييمات بعد'
            : `${number(Math.round(summary.meanResponseMs / 1000))} ثانية`}
        </dd>
        <dt>مستحقة متبقية ضمن الفلتر</dt>
        <dd>{number(summary.matchingDueRemaining)}</dd>
        <dt>مستحقة متبقية في المجموعة كلها</dt>
        <dd>{number(summary.allDueRemaining)}</dd>
        <dt>الاستحقاق القادم</dt>
        <dd>
          {summary.nextDueAt ? dateTime(summary.nextDueAt, timezone) : 'لا توجد مواعيد قادمة'}
        </dd>
      </dl>
      <table>
        <caption>تقييمات الجلسة</caption>
        <thead>
          <tr>
            <th scope="col">التقييم</th>
            <th scope="col">المحاولات</th>
          </tr>
        </thead>
        <tbody>
          {ratings.map((rating, i) => (
            <tr key={rating}>
              <th scope="row">{ratingLabels[rating]}</th>
              <td>{number(summary.ratings[i])}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {summary.filterChanges.length > 0 && (
        <details>
          <summary>تغييرات فلتر الجلسة</summary>
          <ul>
            {summary.filterChanges.map((change, i) => (
              <li key={i}>
                {dateTime(change.at, timezone)}:{' '}
                {change.to === 'all'
                  ? 'كل البطاقات المستحقة'
                  : change.to === 'new'
                    ? 'البطاقات الجديدة'
                    : 'بطاقات سبق تقييمها'}
              </li>
            ))}
          </ul>
        </details>
      )}
    </>
  );
}
