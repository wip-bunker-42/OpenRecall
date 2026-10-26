import { act, renderHook } from '@testing-library/react';
import { useReviewSession } from './useReviewSession';
import { fixtureApi } from '../../test/mockApi';
import { questionFixture } from '../../test/reviewFixtures';
it('bounds far-future wakeups and observes arrivals without preempting the presentation', async () => {
  vi.useFakeTimers();
  const calls: string[] = [];
  let reads = 0;
  const api = fixtureApi((r) => {
    calls.push(r.kind);
    if (r.kind === 'readSession')
      return {
        kind: r.kind,
        payload: {
          ...questionFixture,
          payload: {
            ...questionFixture.payload,
            progress: { current: 1, total: ++reads, completed: 0 },
          },
        },
      };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
  });
  try {
    const { result, unmount } = renderHook(() => useReviewSession(api, 'session-1'));
    await act(async () => {
      await Promise.resolve();
    });
    const original = result.current.view;
    expect(original?.state).toBe('question');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(60000);
    });
    expect(calls.filter((k) => k === 'readSession').length).toBeGreaterThan(1);
    expect(calls.filter((k) => k === 'readSession').length).toBeLessThanOrEqual(6);
    expect(
      result.current.view?.state === 'question' && result.current.view.payload.presentationId,
    ).toBe('presentation-1');
    expect(
      result.current.view?.state === 'question' && result.current.view.payload.progress.total,
    ).toBeGreaterThan(1);
    unmount();
    const count = calls.filter((k) => k === 'readSession').length;
    await vi.advanceTimersByTimeAsync(120000);
    expect(calls.filter((k) => k === 'readSession').length).toBe(count);
  } finally {
    vi.useRealTimers();
  }
});
it('serializes focus activity and never reactivates an inactive window on a heartbeat', async () => {
  const focus = vi.spyOn(document, 'hasFocus').mockReturnValue(true);
  const activity: boolean[] = [];
  const api = fixtureApi((r) => {
    if (r.kind === 'readSession') return { kind: r.kind, payload: questionFixture };
    if (r.kind === 'setSessionActivity') {
      activity.push(r.payload.active);
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    }
  });
  vi.useFakeTimers();
  try {
    const { unmount } = renderHook(() => useReviewSession(api, 'session-1'));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(activity.at(-1)).toBe(true);
    await act(async () => {
      window.dispatchEvent(new Event('blur'));
      await vi.advanceTimersByTimeAsync(15000);
    });
    expect(activity.at(-1)).toBe(false);
    await act(async () => {
      window.dispatchEvent(new Event('focus'));
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(activity.at(-1)).toBe(true);
    unmount();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(activity.at(-1)).toBe(false);
  } finally {
    focus.mockRestore();
    vi.useRealTimers();
  }
});
