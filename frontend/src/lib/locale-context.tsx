import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';

import { translate, categoryLabel as translateCategory, type Locale } from '@/lib/i18n';

const STORAGE_KEY = 'jury-assistant-locale';

function detectDefaultLocale(): Locale {
  if (typeof navigator === 'undefined') return 'tr';
  return navigator.language.toLowerCase().startsWith('tr') ? 'tr' : 'en';
}

interface LocaleContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string, vars?: Record<string, string>) => string;
  categoryLabel: (category: string) => string;
}

const LocaleContext = createContext<LocaleContextValue | null>(null);

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>('tr');

  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY) as Locale | null;
    setLocaleState(stored ?? detectDefaultLocale());
  }, []);

  function setLocale(next: Locale) {
    setLocaleState(next);
    localStorage.setItem(STORAGE_KEY, next);
  }

  const value: LocaleContextValue = {
    locale,
    setLocale,
    t: (key, vars) => translate(locale, key, vars),
    categoryLabel: (category) => translateCategory(locale, category),
  };

  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocale() {
  const ctx = useContext(LocaleContext);
  if (!ctx) throw new Error('useLocale must be used within LocaleProvider');
  return ctx;
}
