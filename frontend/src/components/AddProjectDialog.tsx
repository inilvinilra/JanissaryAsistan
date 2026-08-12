import * as React from 'react';
import { useState } from 'react';

import { getCompetitions, uploadProject, type CategoryTemplate, type Competition, type Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';

export function AddProjectDialog({
  categories,
  defaultCategory,
  onCreated,
}: {
  categories: CategoryTemplate[];
  defaultCategory: string;
  onCreated: (project: Project) => void;
}) {
  const { t, categoryLabel } = useLocale();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState('');
  const [category, setCategory] = useState(defaultCategory);
  const [competitions, setCompetitions] = useState<Competition[]>([]);
  const [competitionId, setCompetitionId] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  React.useEffect(() => {
    if (!open) return;
    getCompetitions()
      .then((items) => {
        setCompetitions(items);
        setCompetitionId((current) => current || String(items[0]?.id ?? ''));
      })
      .catch((reason) => setError((reason as Error).message));
  }, [open]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!file) {
      setError(t('chooseFileError'));
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      if (!competitionId) {
        setError('Select a competition before uploading a project.');
        return;
      }
      const project = await uploadProject(name, category, Number(competitionId), file);
      onCreated(project);
      setOpen(false);
      setName('');
      setFile(null);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>{t('addProject')}</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('dialogTitle')}</DialogTitle>
          <DialogDescription>{t('dialogDescription')}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="project-name">{t('fieldName')}</Label>
            <Input id="project-name" value={name} onChange={(e) => setName(e.target.value)} required />
          </div>

          <div className="space-y-2">
            <Label htmlFor="project-competition">Competition</Label>
            <Select value={competitionId} onValueChange={setCompetitionId}>
              <SelectTrigger id="project-competition" className="w-full"><SelectValue placeholder="Select a competition" /></SelectTrigger>
              <SelectContent>{competitions.map((competition) => <SelectItem key={competition.id} value={String(competition.id)}>{competition.name}</SelectItem>)}</SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="project-category">{t('fieldCategory')}</Label>
            <Select value={category} onValueChange={setCategory}>
              <SelectTrigger id="project-category" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {categories.map((cat) => (
                  <SelectItem key={cat.category} value={cat.category}>
                    {categoryLabel(cat.category)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="project-file">{t('fieldFile')}</Label>
            <Input
              id="project-file"
              type="file"
              accept=".pdf,.txt,.md,.markdown"
              onChange={(e) => setFile(e.target.files?.[0] ?? null)}
              required
            />
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button type="submit" disabled={submitting}>
              {submitting ? t('uploading') : t('upload')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
