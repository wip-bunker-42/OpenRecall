import { browser, $, expect, click, heading, audit } from './helpers';
describe('Windows WebView2 keyboard and native focus lifecycle', () => {
  it('preserves dialog focus/escape and restores focus after its own window hides and regains focus', async () => {
    await click('المجموعات');
    await heading('مجموعاتي');
    const create = await $('button=إنشاء مجموعة');
    await create.click();
    await expect($('dialog input')).toBeFocused();
    await browser.keys('Escape');
    await expect($('dialog')).not.toExist();
    await expect(create).toBeFocused();
    const result = await browser.executeAsync((done) => {
      const tauri = (
        window as unknown as {
          __TAURI__: {
            window: {
              getCurrentWindow: () => {
                hide: () => Promise<void>;
                show: () => Promise<void>;
                setFocus: () => Promise<void>;
              };
            };
            webview: { getCurrentWebview: () => { setFocus: () => Promise<void> } };
          };
        }
      ).__TAURI__;
      const win = tauri.window.getCurrentWindow();
      const webview = tauri.webview.getCurrentWebview();
      let blurred = 0,
        focused = 0;
      const blur = () => blurred++,
        focus = () => focused++;
      void (async () => {
        const waitFor = async (condition: () => boolean) => {
          const limit = performance.now() + 5000;
          while (!condition() && performance.now() < limit)
            await new Promise((resolve) => setTimeout(resolve, 50));
          if (!condition()) throw new Error('Native focus transition was not observed');
        };
        try {
          await win.setFocus();
          await webview.setFocus();
          await waitFor(() => document.hasFocus());
          window.addEventListener('blur', blur);
          window.addEventListener('focus', focus);
          await win.hide();
          await waitFor(() => blurred > 0);
          await win.show();
          await win.setFocus();
          await webview.setFocus();
          await waitFor(() => focused > 0 && document.hasFocus());
          done({ blurred, focused, active: document.activeElement?.textContent });
        } catch (error) {
          done({ error: String(error), blurred, focused });
        } finally {
          window.removeEventListener('blur', blur);
          window.removeEventListener('focus', focus);
        }
      })();
    });
    expect(result).toEqual(
      expect.objectContaining({
        blurred: expect.any(Number),
        focused: expect.any(Number),
        active: 'إنشاء مجموعة',
      }),
    );
    expect((result as { blurred: number }).blurred).toBeGreaterThan(0);
    expect((result as { focused: number }).focused).toBeGreaterThan(0);
    await expect(create).toBeFocused();
    await audit('native-refocused');
  });
});
