import { createContext, useContext, useCallback, useId } from 'react';
export const UnsavedContext = createContext<{
  dirty?: boolean;
  setDirty: (value: boolean, source?: string) => void;
}>({
  dirty: false,
  setDirty: () => {},
});
export function useUnsaved() {
  const context = useContext(UnsavedContext);
  const source = useId();
  const setDirty = useCallback(
    (value: boolean) => context.setDirty(value, source),
    [context.setDirty, source],
  );
  return { setDirty };
}
