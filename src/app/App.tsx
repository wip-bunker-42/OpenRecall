import { useCallback, useEffect, useLayoutEffect, useState, useRef } from 'react';
import type { Api, NativeApi } from '../shared/api';
import type { Bootstrap } from '../shared/generated/Bootstrap';
import type { RestoreReport } from '../shared/generated/RestoreReport';
import type { Route } from './navigation';
import { LibraryPage } from '../features/library/LibraryPage';
import { FormErrors } from '../components/FormErrors';
import { focusHeading } from '../shared/focus';
import { DeckPage } from '../features/cards/DeckPage';
import { Dialog } from '../components/Dialog';
import { UnsavedContext } from './unsaved';
import { ReviewPage } from '../features/review/ReviewPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { ProfilesPage } from '../features/profiles/ProfilesPage';
import { StatusRegion } from '../components/StatusRegion';
export function App({ api, native }: { api: Api; native: NativeApi }) {
  const [bootstrap, setBootstrap] = useState<Bootstrap | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [restored, setRestored] = useState<RestoreReport | null>(null);
  const [route, setRoute] = useState<Route>({ page: 'library' });
  const [dirty, setHasDirty] = useState(false);
  const dirtySources = useRef(new Set<string>());
  const setDirty = useCallback((value: boolean, source = 'default') => {
    if (value) dirtySources.current.add(source);
    else dirtySources.current.delete(source);
    setHasDirty(dirtySources.current.size > 0);
  }, []);
  const [pending, setPending] = useState<Route | null>(null);
  const [reviewBusy, setReviewBusy] = useState(false);
  const [navigationBusy, setNavigationBusy] = useState(false);
  function go(next: Route) {
    if (reviewBusy || navigationBusy) return;
    setNavigationBusy(true);
    void (async () => {
      try {
        if (
          route.page === 'review' &&
          (next.page !== 'review' || next.sessionId !== route.sessionId)
        )
          await api.call('closeSession', { sessionId: route.sessionId });
        setRoute(next);
        if (next.page === 'library') await refresh();
      } catch (e) {
        setError(e);
      } finally {
        setNavigationBusy(false);
      }
    })();
  }
  function navigate(next: Route) {
    if (dirtySources.current.size > 0) setPending(next);
    else go(next);
  }
  const navigation = useRef(navigate);
  useLayoutEffect(() => {
    navigation.current = navigate;
  });
  const requestNavigation = useCallback((next: Route) => navigation.current(next), []);
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
    void native.setEditState({ unsavedChanges: dirty }).catch(setError);
  }, [dirty, native]);
  useEffect(() => {
    if (!dirty) return;
    const guard = (e: BeforeUnloadEvent) => {
      e.preventDefault();
    };
    window.addEventListener('beforeunload', guard);
    return () => window.removeEventListener('beforeunload', guard);
  }, [dirty]);
  return (
    <UnsavedContext.Provider value={{ setDirty, dirty }}>
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
                disabled={reviewBusy || navigationBusy}
                aria-current={route.page === 'library' ? 'page' : undefined}
                onClick={() => navigate({ page: 'library' })}
              >
                المجموعات
              </button>
              <button
                disabled={reviewBusy || navigationBusy}
                aria-current={route.page === 'profiles' ? 'page' : undefined}
                onClick={() => navigate({ page: 'profiles' })}
              >
                نماذج التحسين
              </button>
              <button
                disabled={reviewBusy || navigationBusy}
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
            <StatusRegion message={restored ? 'تمت استعادة النسخة الاحتياطية.' : ''} />
            {restored && (
              <div className="result">
                <p>حُفظت نسخة أمان من المكتبة السابقة قبل الاستعادة.</p>
                <p dir="auto">{restored.safetyBackup.path}</p>
                <button
                  onClick={() =>
                    void native
                      .revealExport({ exportId: restored.safetyBackup.exportId })
                      .catch(setError)
                  }
                >
                  إظهار نسخة الأمان السابقة للاستعادة
                </button>
                <button onClick={() => setRestored(null)}>إغلاق إشعار الاستعادة</button>
              </div>
            )}
            {!bootstrap ? (
              <>
                <h1 tabIndex={-1}>OpenRecall</h1>
                <p role="status">جارٍ فتح مكتبتك…</p>
                {error && (
                  <button onClick={() => void refresh().catch(setError)}>إعادة المحاولة</button>
                )}
              </>
            ) : route.page === 'library' ? (
              <LibraryPage
                bootstrap={bootstrap}
                api={api}
                navigate={requestNavigation}
                refresh={refresh}
              />
            ) : route.page === 'deck' ? (
              <DeckPage
                key={route.deckId}
                deckId={route.deckId}
                api={api}
                native={native}
                navigate={requestNavigation}
                pageSize={bootstrap.settings.pageSize}
                timezone={bootstrap.settings.timezone}
              />
            ) : route.page === 'review' ? (
              <ReviewPage
                key={route.sessionId}
                sessionId={route.sessionId}
                deckId={route.deckId}
                api={api}
                navigate={requestNavigation}
                onBusyChange={setReviewBusy}
                externalBusy={navigationBusy}
                shortcutsEnabled={bootstrap.settings.shortcutsEnabled}
                timezone={bootstrap.settings.timezone}
              />
            ) : route.page === 'settings' ? (
              <SettingsPage
                api={api}
                native={native}
                navigate={requestNavigation}
                onSaved={refresh}
                onRestored={async (report) => {
                  setRestored(report);
                  await refresh();
                  go({ page: 'library' });
                }}
              />
            ) : (
              <ProfilesPage api={api} navigate={requestNavigation} />
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
                  dirtySources.current.clear();
                  setHasDirty(false);
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
