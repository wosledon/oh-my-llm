import { create } from 'zustand';
import { zh } from './zh';
import { en } from './en';
import type { Translation } from './zh';

type Lang = 'zh' | 'en';

const translations: Record<Lang, Translation> = { zh, en };

function getStoredLang(): Lang {
  const stored = localStorage.getItem('oh-my-llm-lang');
  if (stored === 'zh' || stored === 'en') return stored;
  return 'zh';
}

interface I18nState {
  lang: Lang;
  t: Translation;
  setLang: (lang: Lang) => void;
}

export const useI18n = create<I18nState>((set) => ({
  lang: getStoredLang(),
  t: translations[getStoredLang()],
  setLang: (lang) => {
    localStorage.setItem('oh-my-llm-lang', lang);
    set({ lang, t: translations[lang] });
  },
}));

export type { Translation };
