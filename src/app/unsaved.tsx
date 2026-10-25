import { createContext, useContext } from 'react';
export const UnsavedContext = createContext<{ setDirty: (value: boolean) => void }>({
  setDirty: () => {},
});
export function useUnsaved() {
  return useContext(UnsavedContext);
}
