import type { TrainingEligibility } from '../../shared/generated/TrainingEligibility';
import { number } from '../../shared/format';
export function EligibilityTable({ eligibility }: { eligibility: TrainingEligibility }) {
  const rows: [string, number, number][] = [
    ['ملاحظات قابلة للتدريب بعد التقييم الأول', eligibility.usableObservations, 400],
    ['بطاقات مختلفة', eligibility.distinctCards, 50],
    ['أيام مذاكرة مختلفة', eligibility.studyDays, 7],
    ['نتائج نسيت', eligibility.againCount, 5],
    ['نتائج تذكر', eligibility.recalledCount, 5],
    ['أمثلة تدريب', eligibility.trainCount, 64],
    ['ملاحظات اختبار', eligibility.holdoutCount, 80],
    ['بطاقات في الاختبار', eligibility.holdoutCards, 10],
  ];
  return (
    <>
      <p>
        {eligibility.eligible
          ? 'البيانات كافية لبدء المقارنة. تطبيق الأوزان يظل قرارًا مستقلًا بعد ظهور النتيجة.'
          : 'البيانات غير كافية للتدريب. أكمل الشروط التالية بالمراجعة المعتادة.'}
      </p>
      <div className="table-scroll">
        <table>
          <caption>شروط السماح بالتدريب</caption>
          <thead>
            <tr>
              <th scope="col">الشرط</th>
              <th scope="col">المتوفر والحد الأدنى</th>
              <th scope="col">الحالة</th>
            </tr>
          </thead>
          <tbody>
            {rows.map(([label, value, min]) => (
              <tr key={label}>
                <th scope="row">{label}</th>
                <td>{`${number(value)} من ${number(min)}`}</td>
                <td>{value >= min ? 'مكتمل' : 'غير مكتمل'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <table>
        <caption>مساهمات المجموعات في بيانات التدريب</caption>
        <thead>
          <tr>
            <th scope="col">المجموعة</th>
            <th scope="col">الملاحظات</th>
            <th scope="col">البطاقات</th>
          </tr>
        </thead>
        <tbody>
          {eligibility.contributions.map((c) => (
            <tr key={c.deckId}>
              <th scope="row">
                <bdi>{c.deckName}</bdi>
              </th>
              <td>{number(c.observations)}</td>
              <td>{number(c.cards)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="muted">
        هذه الأعداد تخص المجموعات المرتبطة بهذا النموذج فقط، وليست سجل المستخدم كله.
      </p>
    </>
  );
}
