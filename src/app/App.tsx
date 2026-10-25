import { useCallback, useEffect, useState } from 'react';
import type { Api, NativeApi } from '../shared/api';
import type { Bootstrap } from '../shared/generated/Bootstrap';
import type { Route } from './navigation';
import { LibraryPage } from '../features/library/LibraryPage';
import { FormErrors } from '../components/FormErrors';
import { focusHeading } from '../shared/focus';
import { DeckPage } from '../features/cards/DeckPage';
import { Dialog } from '../components/Dialog';
import { UnsavedContext } from './unsaved';
export function App({ api, native }: { api: Api; native: NativeApi }) {
  const [bootstrap, setBootstrap] = useState<Bootstrap | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [route, setRoute] = useState<Route>({ page: 'library' });
  const [dirty, setDirty] = useState(false);
  const [pending, setPending] = useState<Route | null>(null);
  function go(next: Route) {
    setRoute(next);
    if (next.page === 'library') void refresh().catch(setError);
  }
  function navigate(next: Route) {
    if (dirty) setPending(next);
    else go(next);
  }
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
  useEffect(() => {
    if (!dirty) return;
    const guard = (e: BeforeUnloadEvent) => {
      e.preventDefault();
    };
    window.addEventListener('beforeunload', guard);
    return () => window.removeEventListener('beforeunload', guard);
  }, [dirty]);
  return (
    <UnsavedContext.Provider value={{ setDirty }}>
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
                onClick={() => navigate({ page: 'library' })}
              >
                المجموعات
              </button>
              <button
                aria-current={route.page === 'profiles' ? 'page' : undefined}
                onClick={() => navigate({ page: 'profiles' })}
              >
                نماذج التحسين
              </button>
              <button
                aria-current={route.page === 'settings' ? 'page' : undefined}
                onClick={() => navigate({ page: 'settings' })}
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
              <LibraryPage bootstrap={bootstrap} api={api} navigate={navigate} refresh={refresh} />
            ) : route.page === 'deck' ? (
              <DeckPage
                key={route.deckId}
                deckId={route.deckId}
                api={api}
                native={native}
                navigate={navigate}
                pageSize={bootstrap.settings.pageSize}
              />
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
        {pending && (
          <Dialog title="تغييرات غير محفوظة" onDismiss={() => setPending(null)}>
            <p>احفظ تعديلاتك قبل مغادرة الصفحة، أو تجاهلها للمتابعة.</p>
            <div className="actions">
              <button data-autofocus onClick={() => setPending(null)}>
                متابعة التعديل
              </button>
              <button
                className="danger"
                onClick={() => {
                  setDirty(false);
                  setPending(null);
                  go(pending);
                }}
              >
                تجاهل التغييرات والمتابعة
              </button>
            </div>
          </Dialog>
        )}
      </>
    </UnsavedContext.Provider>
  );
}
