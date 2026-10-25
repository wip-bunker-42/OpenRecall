export function focusHeading(root: HTMLElement | null = document.querySelector('main')) {
  root?.querySelector<HTMLElement>('h1')?.focus();
}
