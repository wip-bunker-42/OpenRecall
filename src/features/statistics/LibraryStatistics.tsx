import { useCallback, useState } from 'react';
import type { Api } from '../../shared/api';
import { FormErrors } from '../../components/FormErrors';
import { number } from '../../shared/format';
import { useStatistics } from './useStatistics';
export function LibraryStatistics({
  api,
  revision,
  timezone,
}: {
  api: Api;
  revision: string;
  timezone: string;
}) {
  const read = useCallback(() => api.call('getLibraryStatistics', {}), [api]);
  const { stats, error, refresh } = useStatistics(read, `${revision}:${timezone}`);
  const [history, setHistory] = useState(false);
  const [requestedPage, setPage] = useState(1);
  const daily = stats?.dailyHistory.slice().reverse() ?? [];
  const page = Math.min(requestedPage, Math.max(1, Math.ceil(daily.length / 20)));
  const day = (date: string) =>
    new Intl.DateTimeFormat('ar-EG', { dateStyle: 'medium', timeZone: 'UTC' }).format(
      new Date(`${date}T12:00:00Z`),
    );
  return (
    <>
      <FormErrors error={error} focus={false} />
      {!stats ? (
        <p role="status">جارٍ حساب إحصاءات المكتبة…</p>
      ) : (
        <>
          <dl className="statistics-list">
            <dt>انتهت مراجعاتها اليوم</dt>
            <dd>{number(stats.today.completedForDay)}</dd>
            <dt>منها كلمات الإملاء</dt>
            <dd>{number(stats.spellingCompletedToday)}</dd>
          </dl>
          <p>
            تُحسب البطاقة من آخر تقييم لها في يوم المذاكرة إذا نقل استحقاقها إلى يوم لاحق. مراجعة
            البطاقة الأصلية والتدرب على كلمتها مهمتان مستقلتان في الإجمالي.
          </p>
          <details onToggle={(e) => setHistory(e.currentTarget.open)}>
            <summary>الإنجاز في الأيام السابقة</summary>
            {history &&
              (daily.length === 0 ? (
                <p>لا توجد مراجعات محفوظة بعد.</p>
              ) : (
                <>
                  <div
                    className="table-scroll"
                    role="region"
                    aria-label="جدول الإنجاز اليومي"
                    tabIndex={0}
                  >
                    <table>
                      <caption>الإنجاز حسب يوم المذاكرة</caption>
                      <thead>
                        <tr>
                          <th scope="col">يوم المذاكرة</th>
                          <th scope="col">انتهت مراجعاتها في اليوم</th>
                        </tr>
                      </thead>
                      <tbody>
                        {daily.slice((page - 1) * 20, page * 20).map((row) => (
                          <tr key={row.date}>
                            <th scope="row">
                              <time dateTime={row.date}>{day(row.date)}</time>
                            </th>
                            <td>{number(row.statistics.completedForDay)}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  <div className="actions">
                    <button disabled={page <= 1} onClick={() => setPage(page - 1)}>
                      الأيام الأحدث
                    </button>
                    <button disabled={page * 20 >= daily.length} onClick={() => setPage(page + 1)}>
                      الأيام الأقدم
                    </button>
                  </div>
                </>
              ))}
          </details>
        </>
      )}
      <button onClick={() => void refresh()}>تحديث إحصاءات المكتبة</button>
    </>
  );
}
