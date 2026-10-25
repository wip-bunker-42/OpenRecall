import { useCallback, useEffect, useState } from 'react';
import type { Api, NativeApi } from '../shared/api';
import type { Bootstrap } from '../shared/generated/Bootstrap';
import type { Route } from './navigation';
import { LibraryPage } from '../features/library/LibraryPage';
import { FormErrors } from '../components/FormErrors';
import { focusHeading } from '../shared/focus';
export function App({ api, native }: { api: Api; native: NativeApi }) {
  const [bootstrap, setBootstrap] = useState<Bootstrap | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [route, setRoute] = useState<Route>({ page: 'library' });
  const refresh = useCallback(async () => {
    setBootstrap(await api.call('bootstrap', {}));
  }, [api]);
  useEffect(() => {
    document.documentElement.lang = 'ar';
    document.documentElement.dir = 'rtl';
    void refresh().catch(setError);
  }, [refresh]);
  useEffect(() => {
    focusHeading();
  }, [route, Boolean(bootstrap)]);
  useEffect(() => {
    if (bootstrap) {
      document.documentElement.style.setProperty(
        '--text-scale',
        String(bootstrap.settings.textScale),
      );
      document.documentElement.dataset.motion = bootstrap.settings.reducedMotion;
    }
  }, [bootstrap]);
  void native;
  return (
    <>
      <a className="skip" href="#main">
        تجاوز إلى المحتوى
      </a>
      <div className="app-shell">
        <aside className="rail">
          <div className="wordmark" dir="ltr">
            OpenRecall
            <span className="brand-mark" aria-hidden="true">
              ◒
            </span>
          </div>
          <nav aria-label="التنقل الرئيسي">
            <button
              aria-current={route.page === 'library' ? 'page' : undefined}
              onClick={() => setRoute({ page: 'library' })}
            >
              المجموعات
            </button>
            <button
              aria-current={route.page === 'profiles' ? 'page' : undefined}
              onClick={() => setRoute({ page: 'profiles' })}
            >
              نماذج التحسين
            </button>
            <button
              aria-current={route.page === 'settings' ? 'page' : undefined}
              onClick={() => setRoute({ page: 'settings' })}
            >
              الإعدادات
            </button>
          </nav>
          <p className="rail-note">
            مذاكرتك محفوظة
            <br />
            على جهازك
          </p>
        </aside>
        <main id="main" tabIndex={-1}>
          <FormErrors error={error} />
          {!bootstrap ? (
            <>
              <h1 tabIndex={-1}>OpenRecall</h1>
              <p role="status">جارٍ فتح مكتبتك…</p>
              {error && (
                <button onClick={() => void refresh().catch(setError)}>إعادة المحاولة</button>
              )}
            </>
          ) : route.page === 'library' ? (
            <LibraryPage bootstrap={bootstrap} api={api} navigate={setRoute} refresh={refresh} />
          ) : (
            <h1 tabIndex={-1}>
              {route.page === 'settings'
                ? 'الإعدادات'
                : route.page === 'profiles'
                  ? 'نماذج التحسين'
                  : route.page === 'review'
                    ? 'المراجعة'
                    : 'المجموعة'}
            </h1>
          )}
        </main>
      </div>
    </>
  );
}
