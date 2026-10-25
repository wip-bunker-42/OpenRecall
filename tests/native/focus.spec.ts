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
      const win = (
        window as unknown as {
          __TAURI__: {
            window: {
              getCurrentWindow: () => {
                hide: () => Promise<void>;
                show: () => Promise<void>;
                setFocus: () => Promise<void>;
              };
            };
          };
        }
      ).__TAURI__.window.getCurrentWindow();
      let blurred = 0,
        focused = 0;
      const blur = () => blurred++,
        focus = () => focused++;
      window.addEventListener('blur', blur);
      window.addEventListener('focus', focus);
      void (async () => {
        await win.hide();
        await new Promise((resolve) => setTimeout(resolve, 250));
        await win.show();
        await win.setFocus();
        await new Promise((resolve) => setTimeout(resolve, 250));
        window.removeEventListener('blur', blur);
        window.removeEventListener('focus', focus);
        done({ blurred, focused, active: document.activeElement?.textContent });
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
