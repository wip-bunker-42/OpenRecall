import { createApi } from './api';
it('verifies response kinds and sends typed requests', async () => {
  const transport = vi.fn().mockResolvedValue({ kind: 'listDecks', payload: [] });
  const api = createApi(transport);
  expect(await api.call('listDecks', {})).toEqual([]);
  expect(transport).toHaveBeenCalledWith({ kind: 'listDecks', payload: {} });
  transport.mockResolvedValue({ kind: 'deleteDeck', payload: { id: 'x' } });
  await expect(api.call('listDecks', {})).rejects.toMatchObject({ code: 'responseMismatch' });
});
