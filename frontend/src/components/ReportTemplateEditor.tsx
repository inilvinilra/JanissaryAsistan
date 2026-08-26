import { useEffect, useState, type FormEvent } from 'react';
import { Plus, Trash2 } from 'lucide-react';

import { getReportTemplate, getSupportedLanguages, saveReportTemplate, type TemplateSection } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

const EMPTY_SECTION: TemplateSection = { key: '', title: '', aliases: [], min_words: 0, required: true };

function slugify(title: string): string {
  return title
    .toLowerCase()
    .replace(/ı/g, 'i').replace(/ğ/g, 'g').replace(/ü/g, 'u')
    .replace(/ş/g, 's').replace(/ö/g, 'o').replace(/ç/g, 'c')
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
}

export function ReportTemplateEditor({ competitionId }: { competitionId: number }) {
  const { t } = useLocale();
  const { showToast } = useToast();
  const [name, setName] = useState('');
  const [expectedLanguage, setExpectedLanguage] = useState('Turkish');
  const [minWords, setMinWords] = useState(0);
  const [maxWords, setMaxWords] = useState(0);
  const [sections, setSections] = useState<TemplateSection[]>([]);
  const [version, setVersion] = useState<number | null>(null);
  const [saving, setSaving] = useState(false);
  const [languages, setLanguages] = useState<string[]>([]);

  useEffect(() => {
    getSupportedLanguages().then(setLanguages).catch(() => setLanguages([]));
  }, []);

  useEffect(() => {
    let active = true;
    getReportTemplate(competitionId)
      .then((template) => {
        if (!active) return;
        setName(template.name);
        setExpectedLanguage(template.expected_language);
        setMinWords(template.min_words);
        setMaxWords(template.max_words);
        setSections(template.sections);
        setVersion(template.version);
      })
      .catch(() => {
        if (!active) return;
        setName('');
        setSections([{ ...EMPTY_SECTION }]);
        setVersion(null);
      });
    return () => { active = false; };
  }, [competitionId]);

  function updateSection(index: number, patch: Partial<TemplateSection>) {
    setSections((current) => current.map((section, i) => (i === index ? { ...section, ...patch } : section)));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    try {
      const payload = sections.map((section) => ({
        ...section,
        key: section.key.trim() || slugify(section.title),
      }));
      const saved = await saveReportTemplate(competitionId, {
        name,
        expected_language: expectedLanguage,
        min_words: minWords,
        max_words: maxWords,
        sections: payload,
      });
      setVersion(saved.version);
      setSections(saved.sections);
      showToast(t('templateSaved', { version: String(saved.version) }));
    } catch (error) {
      showToast(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          {t('templateEditorTitle')}
          {version !== null && <span className="text-muted-foreground font-normal"> · v{version}</span>}
        </CardTitle>
        <CardDescription>{t('templateEditorDescription')}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={submit} className="space-y-3">
          <div className="grid gap-2 sm:grid-cols-2">
            <label className="space-y-1">
              <span className="text-muted-foreground text-xs">{t('templateNameLabel')}</span>
              <Input value={name} onChange={(event) => setName(event.target.value)} required />
            </label>
            <label className="space-y-1">
              <span className="text-muted-foreground text-xs">{t('templateLanguageLabel')}</span>
              <select
                value={expectedLanguage}
                onChange={(event) => setExpectedLanguage(event.target.value)}
                className="border-input bg-background h-9 w-full rounded-md border px-3 text-sm"
              >
                <option value="Any">{t('templateAnyLanguage')}</option>
                {(languages.length > 0 ? languages : [expectedLanguage].filter(Boolean)).map((language) => (
                  <option key={language} value={language}>{language}</option>
                ))}
              </select>
            </label>
            <label className="space-y-1">
              <span className="text-muted-foreground text-xs">{t('templateMinWords')}</span>
              <Input type="number" min={0} value={minWords} onChange={(event) => setMinWords(Number(event.target.value))} />
            </label>
            <label className="space-y-1">
              <span className="text-muted-foreground text-xs">{t('templateMaxWords')}</span>
              <Input type="number" min={0} value={maxWords} onChange={(event) => setMaxWords(Number(event.target.value))} />
            </label>
          </div>

          <div className="space-y-2 border-t pt-3">
            <p className="text-muted-foreground text-xs font-semibold tracking-wide uppercase">{t('templateSectionsLabel')}</p>
            {sections.map((section, index) => (
              <div key={index} className="space-y-2 rounded-md border p-2">
                <div className="grid gap-2 sm:grid-cols-[1fr_1fr_6rem]">
                  <Input
                    placeholder={t('templateSectionTitle')}
                    value={section.title}
                    onChange={(event) => updateSection(index, { title: event.target.value })}
                    required
                  />
                  <Input
                    placeholder={t('templateSectionAliases')}
                    value={section.aliases.join(', ')}
                    onChange={(event) => updateSection(index, { aliases: event.target.value.split(',').map((alias) => alias.trim()).filter(Boolean) })}
                  />
                  <Input
                    type="number"
                    min={0}
                    placeholder={t('templateSectionMinWords')}
                    value={section.min_words}
                    onChange={(event) => updateSection(index, { min_words: Number(event.target.value) })}
                  />
                </div>
                <div className="flex items-center justify-between gap-2">
                  <label className="flex items-center gap-2 text-xs">
                    <input
                      type="checkbox"
                      checked={section.required}
                      onChange={(event) => updateSection(index, { required: event.target.checked })}
                    />
                    {t('templateSectionRequired')}
                  </label>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => setSections((current) => current.filter((_, i) => i !== index))}
                  >
                    <Trash2 className="size-3.5" /> {t('templateRemoveSection')}
                  </Button>
                </div>
              </div>
            ))}
            <Button type="button" variant="outline" size="sm" onClick={() => setSections((current) => [...current, { ...EMPTY_SECTION }])}>
              <Plus className="size-3.5" /> {t('templateAddSection')}
            </Button>
          </div>

          <Button type="submit" disabled={saving}>{t('templateSave')}</Button>
        </form>
      </CardContent>
    </Card>
  );
}
