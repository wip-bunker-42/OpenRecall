import { useCallback, useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { DeckStatistics as Statistics } from '../../shared/generated/DeckStatistics';
import type { PeriodStatistics } from '../../shared/generated/PeriodStatistics';
import type { ForecastDay } from '../../shared/generated/ForecastDay';
import { EligibilityTable } from '../profiles/EligibilityTable';
import { FormErrors } from '../../components/FormErrors';
import { number, dateTime } from '../../shared/format';
const percentage = (v: number | null) =>
  v === null
    ? 'لا توجد نتائج'
    : new Intl.NumberFormat('ar-EG', { style: 'percent', maximumFractionDigits: 1 }).format(v);
const seconds = (v: number | null) =>
  v === null ? 'لا توجد تقييمات' : `${number(Math.round(v / 1000))} ثانية`;
const day = (date: string) =>
  new Intl.DateTimeFormat('ar-EG', { dateStyle: 'medium', timeZone: 'UTC' }).format(
    new Date(`${date}T12:00:00Z`),
  );
function Forecast({ caption, rows }: { caption: string; rows: ForecastDay[] }) {
  return (
    <table>
      <caption>{caption}</caption>
      <thead>
        <tr>
          <th scope="col">يوم المذاكرة</th>
          <th scope="col">البطاقات</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.date}>
            <th scope="row">
              <time dateTime={row.date}>{day(row.date)}</time>
            </th>
            <td>{number(row.cards)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
function PeriodRow({ label, value }: { label: string; value: PeriodStatistics }) {
  return (
    <tr>
      <th scope="row">{label}</th>
      <td>{number(value.attempts)}</td>
      <td>{number(value.uniqueCards)}</td>
      <td>{seconds(value.activeMs)}</td>
      <td>{seconds(value.meanResponseMs)}</td>
      <td>{percentage(value.observedRecallRate)}</td>
      {value.ratingCounts.map((count, i) => (
        <td key={i}>{number(count)}</td>
      ))}
    </tr>
  );
}
const Head = () => (
  <thead>
    <tr>
      <th scope="col">الفترة</th>
      <th scope="col">المحاولات</th>
      <th scope="col">البطاقات المختلفة</th>
      <th scope="col">وقت الإجابة الفعلي</th>
      <th scope="col">متوسط الإجابة</th>
      <th scope="col">التذكر المرصود</th>
      <th scope="col">نسيت</th>
      <th scope="col">بصعوبة</th>
      <th scope="col">بشكل طبيعي</th>
      <th scope="col">بسهولة</th>
    </tr>
  </thead>
);
export function DeckStatistics({
  deckId,
  api,
  timezone = 'Africa/Cairo',
  revision,
}: {
  deckId: string;
  api: Api;
  timezone?: string;
  revision?: string;
}) {
  const [stats, setStats] = useState<Statistics | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [forecasts, setForecasts] = useState(false);
  const [history, setHistory] = useState(false);
  const [page, setPage] = useState(1);
  const load = useCallback(
    async () => setStats(await api.call('getDeckStatistics', { deckId })),
    [api, deckId],
  );
  useEffect(() => {
    void load().catch(setError);
  }, [load, revision]);
  if (!stats)
    return (
      <>
        <FormErrors error={error} />
        <p role="status">جارٍ حساب الإحصاءات…</p>
        <button onClick={() => void load().catch(setError)}>تحديث الإحصاءات</button>
      </>
    );
  const periods: [string, PeriodStatistics][] = [
    ['اليوم', stats.today],
    ['آخر ٧ أيام', stats.last7Days],
    ['آخر ٣٠ يومًا', stats.last30Days],
    ['كل الوقت', stats.allTime],
  ];
  const daily = stats.dailyHistory.slice().reverse();
  return (
    <>
      <h3>إحصاءات هذه المجموعة</h3>
      <FormErrors error={error} />
      <dl className="statistics-list">
        <dt>عدد البطاقات</dt>
        <dd>{number(stats.cards)}</dd>
        <dt>عدد الصيغ</dt>
        <dd>{number(stats.forms)}</dd>
        <dt>جديدة</dt>
        <dd>{number(stats.phases.new)}</dd>
        <dt>في التعلّم</dt>
        <dd>{number(stats.phases.learning)}</dd>
        <dt>في إعادة التعلّم</dt>
        <dd>{number(stats.phases.relearning)}</dd>
        <dt>في المراجعة</dt>
        <dd>{number(stats.phases.review)}</dd>
        <dt>مستحقة الآن</dt>
        <dd>{number(stats.dueNow)}</dd>
        <dt>لاحقًا في يوم المذاكرة الحالي</dt>
        <dd>{number(stats.dueLaterToday)}</dd>
        <dt>أقرب استحقاق قادم</dt>
        <dd>{stats.nextDueAt ? dateTime(stats.nextDueAt, timezone) : 'لا توجد مواعيد قادمة'}</dd>
      </dl>
      <p>
        كل بطاقة تُحسب مرة واحدة مهما تعددت صيغها. التذكر المرصود هو نسبة تقييمات الصعوبة والطبيعي
        والسهل من المحاولات؛ «نسيت» لا تُحسب تذكرًا.
      </p>
      <div className="table-scroll" role="region" aria-label="جدول فترات المراجعة" tabIndex={0}>
        <table>
          <caption>المراجعة عبر الفترات</caption>
          <Head />
          <tbody>
            {periods.map(([label, value]) => (
              <PeriodRow key={label} label={label} value={value} />
            ))}
          </tbody>
        </table>
      </div>
      <details onToggle={(e) => setForecasts(e.currentTarget.open)}>
        <summary>توقعات الاستحقاق</summary>
        {forecasts && (
          <>
            <p>
              الأعداد حسب المواعيد الحالية وبداية يوم المذاكرة. تُضاف المتأخرة إلى اليوم الحالي، وقد
              تغيّر المراجعات القادمة هذه التوقعات.
            </p>
            <Forecast caption="الاستحقاقات خلال ٧ أيام" rows={stats.forecast7Days} />
            <Forecast caption="الاستحقاقات خلال ٣٠ يومًا" rows={stats.forecast30Days} />
          </>
        )}
      </details>
      <details onToggle={(e) => setHistory(e.currentTarget.open)}>
        <summary>سجل المراجعات اليومي</summary>
        {history &&
          (daily.length === 0 ? (
            <p>لا توجد مراجعات محفوظة بعد.</p>
          ) : (
            <>
              <div
                className="table-scroll"
                role="region"
                aria-label="جدول المراجعات اليومية"
                tabIndex={0}
              >
                <table>
                  <caption>المراجعات حسب يوم المذاكرة</caption>
                  <Head />
                  <tbody>
                    {daily.slice((page - 1) * 20, page * 20).map((row) => (
                      <PeriodRow key={row.date} label={day(row.date)} value={row.statistics} />
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="actions">
                <button disabled={page === 1} onClick={() => setPage((p) => p - 1)}>
                  الأيام الأحدث
                </button>
                <button disabled={page * 20 >= daily.length} onClick={() => setPage((p) => p + 1)}>
                  الأيام الأقدم
                </button>
              </div>
            </>
          ))}
      </details>
      <h3>أهلية النموذج المشترك</h3>
      <p>
        <bdi>{stats.profileStatus.name}</bdi>. هذه البيانات مجمعة لكل المجموعات المرتبطة بالنموذج،
        وتختلف عن إحصاءات المجموعة أعلاه.
      </p>
      <details>
        <summary>شروط ومساهمات تدريب النموذج</summary>
        <EligibilityTable eligibility={stats.profileStatus.eligibility} />
      </details>
      <button
        onClick={() =>
          void load()
            .then(() => setError(null))
            .catch(setError)
        }
      >
        تحديث الإحصاءات
      </button>
    </>
  );
}
