export type Route =
  | { page: 'library' }
  | { page: 'deck'; deckId: string }
  | { page: 'review'; sessionId: string; deckId?: string }
  | { page: 'settings' }
  | { page: 'profiles' };
export type Navigate = (route: Route) => void;
