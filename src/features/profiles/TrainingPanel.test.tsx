import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TrainingPanel } from './TrainingPanel';
import { fixtureApi } from '../../test/mockApi';
import { profileFixture, eligibilityFixture, readyJobFixture } from '../../test/profileFixtures';
it.each(['ready', 'failure'])(
  'keeps the new job cancellable when an older %s load arrives late',
  async (result) => {
    let release!: (value: unknown) => void;
    const pending = new Promise<unknown>((resolve) => {
      release = resolve;
    });
    const queued = {
      ...readyJobFixture,
      id: 'new-job',
      status: 'queued',
      candidateParameters: null,
    };
    const api = fixtureApi(async (r) => {
      if (r.kind === 'getTrainingEligibility')
        return { kind: r.kind, payload: readyJobFixture.eligibility };
      if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
      if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [readyJobFixture] };
      if (r.kind === 'getTrainingJob') {
        if (r.payload.jobId === readyJobFixture.id) {
          await pending;
          if (result === 'failure') throw { code: 'staleCandidate' };
          return { kind: r.kind, payload: readyJobFixture };
        }
        return { kind: r.kind, payload: queued };
      }
      if (r.kind === 'startTraining') return { kind: r.kind, payload: queued };
    });
    const user = userEvent.setup();
    render(<TrainingPanel profileId="profile-1" api={api} />);
    await user.click(await screen.findByRole('button', { name: 'بدء التحسين' }));
    expect(await screen.findByRole('button', { name: 'إلغاء التدريب' })).toBeEnabled();
    await act(async () => release(undefined));
    expect(screen.getByRole('button', { name: 'إلغاء التدريب' })).toBeEnabled();
    expect(screen.queryByRole('button', { name: 'تطبيق النموذج المحسن' })).not.toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  },
);
it('revalidates a persisted ready candidate before presenting it as applicable', async () => {
  const api = fixtureApi((r) => {
    if (r.kind === 'getTrainingEligibility')
      return { kind: r.kind, payload: readyJobFixture.eligibility };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
    if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [readyJobFixture] };
    if (r.kind === 'getTrainingJob')
      return {
        kind: r.kind,
        payload: {
          ...readyJobFixture,
          status: 'failed',
          error: { code: 'staleCandidate', fields: [], context: null },
        },
      };
  });
  render(<TrainingPanel profileId="profile-1" api={api} />);
  expect(await screen.findByRole('alert')).toHaveTextContent('تغيرت البيانات');
  expect(screen.queryByRole('button', { name: 'تطبيق النموذج المحسن' })).not.toBeInTheDocument();
});
it('polls real job phases and progress, then reports a native training failure', async () => {
  let polls = 0;
  const queued = {
    ...readyJobFixture,
    status: 'queued',
    progress: { current: 0, total: 9, fraction: 0 },
    baselineMetrics: null,
    candidateMetrics: null,
    candidateParameters: null,
  };
  const api = fixtureApi((r) => {
    if (r.kind === 'getTrainingEligibility') return { kind: r.kind, payload: queued.eligibility };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
    if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [queued] };
    if (r.kind === 'getTrainingJob')
      return {
        kind: r.kind,
        payload:
          ++polls === 1
            ? { ...queued, status: 'running', progress: { current: 4, total: 9, fraction: 0.5 } }
            : {
                ...queued,
                status: 'failed',
                error: { code: 'trainingFailed', fields: [], context: null },
              },
      };
  });
  vi.useFakeTimers();
  try {
    const { unmount } = render(<TrainingPanel profileId="profile-1" api={api} />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByRole('progressbar', { name: 'تقدم التدريب' })).toHaveAttribute('value', '0');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(screen.getByRole('progressbar', { name: 'تقدم التدريب' })).toHaveAttribute(
      'value',
      '0.5',
    );
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(screen.getByRole('status')).toHaveTextContent('تعذر إكمال التدريب');
    expect(screen.queryByRole('button', { name: 'تطبيق النموذج المحسن' })).not.toBeInTheDocument();
    unmount();
  } finally {
    vi.useRealTimers();
  }
});
it('starts an eligible local job and cancels it without an apply request', async () => {
  const calls: unknown[] = [];
  const queued = {
    ...readyJobFixture,
    status: 'queued',
    progress: { current: 0, total: 0, fraction: 0 },
    baselineMetrics: null,
    candidateMetrics: null,
    candidateParameters: null,
  };
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'getTrainingEligibility') return { kind: r.kind, payload: queued.eligibility };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
    if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [] };
    if (r.kind === 'startTraining') return { kind: r.kind, payload: queued };
    if (r.kind === 'cancelTraining')
      return { kind: r.kind, payload: { ...queued, status: 'cancelled' } };
  });
  const user = userEvent.setup();
  render(<TrainingPanel profileId="profile-1" api={api} />);
  await user.click(await screen.findByRole('button', { name: 'بدء التحسين' }));
  await user.click(screen.getByRole('button', { name: 'إلغاء التدريب' }));
  expect(screen.getByRole('status')).toHaveTextContent('أُلغي التدريب');
  expect(calls).toContainEqual({ kind: 'cancelTraining', payload: { jobId: 'job-1' } });
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'applyTraining' }));
});
it('blocks insufficient data and separates pooled contribution counts', async () => {
  const api = fixtureApi((r) => {
    if (r.kind === 'getTrainingEligibility') return { kind: r.kind, payload: eligibilityFixture };
    if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [] };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
  });
  render(<TrainingPanel profileId="profile-1" api={api} />);
  expect(await screen.findByRole('button', { name: 'بدء التحسين' })).toBeDisabled();
  expect(screen.getByText(/٣٩٩.*٤٠٠/)).toBeVisible();
  expect(
    screen.getByRole('table', { name: 'مساهمات المجموعات في بيانات التدريب' }),
  ).toHaveTextContent('لغة');
});
it('requires explicit apply after showing baseline, candidate and held-out evaluation scope', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'getTrainingEligibility')
      return { kind: r.kind, payload: readyJobFixture.eligibility };
    if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [readyJobFixture] };
    if (r.kind === 'getTrainingJob') return { kind: r.kind, payload: readyJobFixture };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
    if (r.kind === 'applyTraining')
      return {
        kind: r.kind,
        payload: {
          ...profileFixture,
          activeModelRevision: 'trained-model',
          modelOrigin: 'trained',
        },
      };
  });
  const user = userEvent.setup();
  const applied = vi.fn();
  render(<TrainingPanel profileId="profile-1" api={api} onApplied={applied} />);
  await screen.findByRole('button', { name: 'تطبيق النموذج المحسن' });
  expect(screen.getByRole('table', { name: 'مقارنة الأداء على بيانات الاختبار' })).toBeVisible();
  expect(screen.getByText(/الاختبار يشمل ٨٠/)).toBeVisible();
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'applyTraining' }));
  await user.click(screen.getByRole('button', { name: 'تطبيق النموذج المحسن' }));
  expect(screen.getByRole('dialog')).toHaveTextContent('كل المجموعات المرتبطة');
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', { name: 'تأكيد تطبيق النموذج' }),
  );
  expect(calls).toContainEqual({
    kind: 'applyTraining',
    payload: { jobId: 'job-1', expectedProfileRevision: 'profile-revision' },
  });
  expect(applied).toHaveBeenCalled();
});
it('reports no improvement without applying or changing a model', async () => {
  const calls: unknown[] = [];
  const job = { ...readyJobFixture, status: 'noImprovement', reason: 'metricsNotImproved' };
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'getTrainingEligibility') return { kind: r.kind, payload: job.eligibility };
    if (r.kind === 'listTrainingRuns') return { kind: r.kind, payload: [job] };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
  });
  render(<TrainingPanel profileId="profile-1" api={api} />);
  expect(await screen.findByText(/لم تتحسن النتائج/)).toBeVisible();
  expect(screen.queryByRole('button', { name: 'تطبيق النموذج المحسن' })).not.toBeInTheDocument();
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'applyTraining' }));
});
