import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
afterEach(cleanup);
// Native dialogs are exercised in browser/native tests; JSDOM has no top layer.
HTMLDialogElement.prototype.showModal = function () {
  this.setAttribute('open', '');
  this.querySelector<HTMLElement>('input,button')?.focus();
};
HTMLDialogElement.prototype.close = function () {
  this.removeAttribute('open');
  this.dispatchEvent(new Event('close'));
};
