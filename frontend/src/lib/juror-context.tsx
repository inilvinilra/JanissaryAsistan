import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';

const STORAGE_KEY = 'jury-assistant-juror-name';

interface JurorContextValue {
  jurorName: string;
  setJurorName: (name: string) => void;
}

const JurorContext = createContext<JurorContextValue | null>(null);

// Free-text name only, stored client-side — not an authenticated identity. Good
// enough to attribute a manual reorder to a person in the activity feed without
// building real accounts.
export function JurorProvider({ children }: { children: ReactNode }) {
  const [jurorName, setJurorNameState] = useState('');

  useEffect(() => {
    setJurorNameState(localStorage.getItem(STORAGE_KEY) ?? '');
  }, []);

  function setJurorName(name: string) {
    setJurorNameState(name);
    localStorage.setItem(STORAGE_KEY, name);
  }

  return <JurorContext.Provider value={{ jurorName, setJurorName }}>{children}</JurorContext.Provider>;
}

export function useJuror() {
  const ctx = useContext(JurorContext);
  if (!ctx) throw new Error('useJuror must be used within JurorProvider');
  return ctx;
}
