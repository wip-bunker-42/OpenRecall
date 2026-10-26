import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ProfilesPage } from './ProfilesPage';
import { fixtureApi, deckFixture } from '../../test/mockApi';
import { profileFixture, linkedDecks } from '../../test/profileFixtures';
import { App } from '../../app/App';
import { nativeFixture } from '../../test/mockApi';
it.each(['name', 'config'])(
  'preserves a %s draft when another profile takes one of its decks',
  async (draft) => {
    let assigned = false;
    const first = { ...profileFixture, name: 'النموذج الأول', memberDeckIds: ['deck-1'] };
    const second = {
      ...profileFixture,
      id: 'profile-2',
      name: 'النموذج الثاني',
      revision: 'second',
      memberDeckIds: ['deck-2'],
    };
    const api = fixtureApi((r) => {
      const one = assigned ? { ...first, revision: 'first-after', memberDeckIds: [] } : first;
      const two = assigned
        ? { ...second, revision: 'second-after', memberDeckIds: ['deck-1', 'deck-2'] }
        : second;
      if (r.kind === 'listProfiles') return { kind: r.kind, payload: [one, two] };
      if (r.kind === 'listDecks')
        return {
          kind: r.kind,
          payload: [
            { ...linkedDecks[0], profileId: assigned ? 'profile-2' : first.id },
            { ...linkedDecks[1], profileId: 'profile-2' },
          ],
        };
      if (r.kind === 'getProfile') return { kind: r.kind, payload: two };
      if (r.kind === 'assignProfile') {
        assigned = true;
        return { kind: r.kind, payload: two };
      }
    });
    const user = userEvent.setup();
    render(<App api={api} native={nativeFixture} />);
    await user.click(await screen.findByRole('button', { name: 'نماذج التحسين' }));
    await user.click(await screen.findByText(first.name));
    await user.click(screen.getByText(second.name));
    const one = screen.getByText(first.name).closest('details')!;
    const two = screen.getByText(second.name).closest('details')!;
    if (draft === 'name') {
      await user.clear(within(one).getByLabelText('اسم نموذج التحسين'));
      await user.type(within(one).getByLabelText('اسم نموذج التحسين'), 'مسودة لا تضيع');
    } else {
      await user.click(within(one).getByText('إعدادات التدريب المتقدمة'));
      await user.clear(within(one).getByLabelText('حجم الدفعة'));
      await user.type(within(one).getByLabelText('حجم الدفعة'), '256');
    }
    await user.selectOptions(within(two).getByLabelText('مجموعة لإضافتها إلى النموذج'), 'deck-1');
    await user.click(within(two).getByRole('button', { name: 'ربط المجموعة بالنموذج' }));
    await user.click(
      within(await screen.findByRole('dialog')).getByRole('button', { name: 'تأكيد الربط' }),
    );
    await screen.findByRole('button', { name: 'فصل المجموعة طب عن النموذج' });
    const current = screen.getByText(first.name).closest('details')!;
    if (draft === 'name')
      expect(within(current).getByLabelText('اسم نموذج التحسين')).toHaveValue('مسودة لا تضيع');
    else expect(within(current).getByLabelText('حجم الدفعة')).toHaveValue(256);
    await user.click(screen.getByRole('button', { name: 'المجموعات' }));
    expect(screen.getByRole('dialog', { name: 'تغييرات غير محفوظة' })).toBeVisible();
  },
);
it('keeps unsaved changes in another expanded profile after saving one profile', async () => {
  let renamed = false;
  const first = { ...profileFixture, name: 'النموذج الأول', memberDeckIds: ['deck-1'] };
  const second = {
    ...profileFixture,
    id: 'profile-2',
    revision: 'profile-2-revision',
    name: 'النموذج الثاني',
    memberDeckIds: ['deck-2'],
  };
  const api = fixtureApi((r) => {
    if (r.kind === 'listProfiles')
      return {
        kind: r.kind,
        payload: [
          renamed ? { ...first, name: 'اسم محفوظ', revision: 'saved-revision' } : first,
          second,
        ],
      };
    if (r.kind === 'listDecks')
      return {
        kind: r.kind,
        payload: [linkedDecks[0], { ...linkedDecks[1], profileId: 'profile-2' }],
      };
    if (r.kind === 'renameProfile') {
      renamed = true;
      return { kind: r.kind, payload: { ...first, name: 'اسم محفوظ', revision: 'saved-revision' } };
    }
  });
  const user = userEvent.setup();
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'نماذج التحسين' }));
  const one = await screen.findByText('النموذج الأول');
  const two = await screen.findByText('النموذج الثاني');
  await user.click(one);
  await user.click(two);
  await user.type(within(two.closest('details')!).getByLabelText('اسم نموذج التحسين'), ' تعديل');
  await user.type(within(one.closest('details')!).getByLabelText('اسم نموذج التحسين'), ' تعديل');
  await user.click(
    within(one.closest('details')!).getByRole('button', { name: 'حفظ اسم النموذج' }),
  );
  await screen.findByText('اسم محفوظ');
  await user.click(screen.getByRole('button', { name: 'المجموعات' }));
  expect(screen.getByRole('dialog', { name: 'تغييرات غير محفوظة' })).toBeVisible();
});
it('protects an unsaved configuration from a separate name save', async () => {
  const api = fixtureApi((r) => {
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: linkedDecks };
  });
  const user = userEvent.setup();
  render(<ProfilesPage api={api} navigate={vi.fn()} />);
  await user.click(await screen.findByText('نموذج مشترك'));
  await user.click(screen.getByText('إعدادات التدريب المتقدمة'));
  await user.clear(screen.getByLabelText('حجم الدفعة'));
  await user.type(screen.getByLabelText('حجم الدفعة'), '256');
  expect(screen.getByLabelText('اسم نموذج التحسين')).toBeDisabled();
  await user.click(screen.getByRole('button', { name: 'تجاهل التعديلات غير المحفوظة' }));
  expect(screen.getByLabelText('اسم نموذج التحسين')).toBeEnabled();
  expect(screen.getByLabelText('حجم الدفعة')).toHaveValue(512);
});
it('validates every training value and explicitly saves disabled penalties for shared members', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: linkedDecks };
    if (r.kind === 'saveTrainingConfig')
      return { kind: r.kind, payload: { ...profileFixture, config: r.payload.config } };
  });
  const user = userEvent.setup();
  render(<ProfilesPage api={api} navigate={vi.fn()} />);
  await user.click(await screen.findByText('نموذج مشترك'));
  await user.click(screen.getByText('إعدادات التدريب المتقدمة'));
  const rate = screen.getByLabelText('معدل التعلم (اختياري)');
  await user.type(rate, '0.000001');
  await user.click(screen.getByRole('button', { name: 'حفظ إعدادات التدريب' }));
  expect(screen.getByRole('alert')).toBeVisible();
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'saveTrainingConfig' }));
  for (const [label, value] of [
    ['عدد العصور (اختياري)', '5'],
    ['حجم الدفعة', '256'],
    ['معدل التعلم (اختياري)', '0.005'],
    ['بذرة العشوائية', '123'],
    ['أقصى طول للسجل', '512'],
    ['مضاعف تنظيم L2', '0.5'],
  ]) {
    await user.clear(screen.getByLabelText(label));
    await user.type(screen.getByLabelText(label), value);
  }
  await user.click(screen.getByRole('checkbox', { name: /Schedule Penalties/ }));
  await user.click(screen.getByRole('button', { name: 'حفظ إعدادات التدريب' }));
  expect(screen.getByRole('dialog')).toHaveTextContent('كل المجموعات المرتبطة');
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', { name: 'تأكيد حفظ إعدادات التدريب' }),
  );
  expect(calls).toContainEqual({
    kind: 'saveTrainingConfig',
    payload: {
      profileId: 'profile-1',
      config: {
        numEpochs: 5,
        batchSize: 256,
        learningRate: 0.005,
        seed: 123,
        maxSeqLen: 512,
        gamma: 0.5,
        schedulePenalties: false,
      },
      expectedRevision: 'profile-revision',
    },
  });
});
it('creates an independent profile, resets its weights and deletes only an unused profile', async () => {
  const calls: unknown[] = [];
  const unused = { ...profileFixture, memberDeckIds: [] };
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [unused] };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: [] };
    if (r.kind === 'createProfile' || r.kind === 'resetProfile')
      return { kind: r.kind, payload: unused };
    if (r.kind === 'deleteProfile') return { kind: r.kind, payload: { id: 'profile-1' } };
  });
  const user = userEvent.setup();
  render(<ProfilesPage api={api} navigate={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'إنشاء نموذج تحسين' }));
  await user.type(within(screen.getByRole('dialog')).getByLabelText('اسم نموذج التحسين'), 'مستقل');
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', { name: 'إنشاء النموذج' }),
  );
  expect(calls).toContainEqual({ kind: 'createProfile', payload: { name: 'مستقل' } });
  await user.click(await screen.findByText('نموذج مشترك'));
  await user.click(screen.getByText('تفاصيل النموذج الحالي'));
  await user.click(screen.getByRole('button', { name: 'استعادة أوزان المحرك الافتراضية' }));
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', { name: 'تأكيد استعادة الأوزان' }),
  );
  expect(calls).toContainEqual({
    kind: 'resetProfile',
    payload: { profileId: 'profile-1', expectedRevision: 'profile-revision' },
  });
  await user.click(screen.getByRole('button', { name: 'حذف النموذج' }));
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', { name: 'تأكيد حذف النموذج' }),
  );
  expect(calls).toContainEqual({
    kind: 'deleteProfile',
    payload: { profileId: 'profile-1', expectedRevision: 'profile-revision' },
  });
});
it('keeps long sections closed, exposes every training field and defaults penalties on', async () => {
  const api = fixtureApi((r) => {
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: linkedDecks };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
  });
  const user = userEvent.setup();
  render(<ProfilesPage api={api} navigate={vi.fn()} />);
  await screen.findByText('نموذج مشترك');
  expect(screen.getByRole('heading', { level: 2, name: 'نموذج مشترك' })).toBeVisible();
  expect(screen.queryByRole('button', { name: 'حفظ إعدادات التدريب' })).not.toBeInTheDocument();
  await user.click(screen.getByText('نموذج مشترك'));
  await user.click(await screen.findByText('إعدادات التدريب المتقدمة'));
  expect(screen.getByRole('checkbox', { name: /Schedule Penalties/ })).toBeChecked();
  expect(screen.getByLabelText('عدد العصور (اختياري)')).toHaveAttribute('max', '100');
  expect(screen.getByLabelText('حجم الدفعة')).toHaveValue(512);
  expect(screen.getByLabelText('معدل التعلم (اختياري)')).toHaveAttribute('min', '0.00001');
  expect(screen.getByLabelText('بذرة العشوائية')).toHaveValue(2023);
  expect(screen.getByLabelText('أقصى طول للسجل')).toHaveAttribute('max', '4096');
  expect(screen.getByLabelText('مضاعف تنظيم L2')).toHaveValue(1);
  await user.click(screen.getByText('تفاصيل النموذج الحالي'));
  expect(screen.getByLabelText('أوزان FSRS-7')).toHaveAttribute('readonly');
  expect(screen.getByRole('button', { name: 'حذف النموذج' })).toBeDisabled();
  expect(screen.getByText(/مرتبط بمجموعات/)).toBeVisible();
});
it('confirms shared membership and detaches by inheriting the current model', async () => {
  const calls: unknown[] = [];
  const spare = {
    ...deckFixture,
    id: 'spare',
    name: 'مجموعة مستقلة',
    profileId: 'other',
    revision: 'spare-revision',
  };
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: [...linkedDecks, spare] };
    if (r.kind === 'getProfile' || r.kind === 'assignProfile' || r.kind === 'detachProfile')
      return { kind: r.kind, payload: profileFixture };
  });
  const user = userEvent.setup();
  render(<ProfilesPage api={api} navigate={vi.fn()} />);
  await user.click(await screen.findByText('نموذج مشترك'));
  await user.selectOptions(await screen.findByLabelText('مجموعة لإضافتها إلى النموذج'), 'spare');
  await user.click(screen.getByRole('button', { name: 'ربط المجموعة بالنموذج' }));
  expect(screen.getByRole('dialog')).toHaveTextContent('كل المجموعات المرتبطة');
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'تأكيد الربط' }));
  expect(calls).toContainEqual({
    kind: 'assignProfile',
    payload: {
      deckId: 'spare',
      profileId: 'profile-1',
      expectedDeckRevision: 'spare-revision',
      expectedProfileRevision: 'profile-revision',
    },
  });
  await user.click(await screen.findByRole('button', { name: 'فصل المجموعة طب عن النموذج' }));
  expect(screen.getByRole('dialog')).toHaveTextContent('يحتفظ بالأوزان والإعدادات الحالية');
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'تأكيد الفصل' }));
  expect(calls).toContainEqual(expect.objectContaining({ kind: 'detachProfile' }));
});
