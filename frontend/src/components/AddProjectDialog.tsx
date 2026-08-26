import * as React from 'react';
import { useState } from 'react';

import { getCompetitions, uploadProject, uploadProjects, projectNameFromFile, type CategoryTemplate, type Competition, type Project } from '@/lib/api';
import { chooseDesktopProjectFile, isDesktopApp, reportExtensions } from '@/lib/desktop';
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
  const [files, setFiles] = useState<File[]>([]);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
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

  const bulk = files.length > 1;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (files.length === 0) {
      setError(t('chooseFileError'));
      return;
    }
    if (!competitionId) {
      setError(t('selectCompetitionFirst'));
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      if (!bulk) {
        const project = await uploadProject(name.trim() || projectNameFromFile(files[0].name), category, Number(competitionId), files[0]);
        onCreated(project);
        close();
        return;
      }
      // Each report becomes its own project, named after its file. One report
      // failing its signature check or malware scan must not discard the rest,
      // so failures are collected and reported instead of thrown.
      setProgress({ done: 0, total: files.length });
      const outcome = await uploadProjects(files, category, Number(competitionId), (done, total) => setProgress({ done, total }));
      outcome.created.forEach(onCreated);
      if (outcome.failed.length > 0) {
        setError(t('bulkUploadPartial', { ok: String(outcome.created.length), failed: String(outcome.failed.length) })
          + ' ' + outcome.failed.map((item) => `${item.fileName}: ${item.reason}`).join(' · '));
        setFiles([]);
      } else {
        close();
      }
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSubmitting(false);
      setProgress(null);
    }
  }

  function close() {
    setOpen(false);
    setName('');
    setFiles([]);
    setError(null);
  }

  async function chooseFile() {
    const selected = await chooseDesktopProjectFile(reportExtensions);
    if (selected) setFiles([selected]);
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (next ? setOpen(true) : close())}>
      <DialogTrigger asChild>
        <Button>{t('addProject')}</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('dialogTitle')}</DialogTitle>
          <DialogDescription>{t('dialogDescription')}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {!bulk && <div className="space-y-2">
            <Label htmlFor="project-name">{t('fieldName')}</Label>
            <Input id="project-name" value={name} onChange={(e) => setName(e.target.value)} required />
          </div>}
          {bulk && <p className="rounded-md bg-muted/50 p-2.5 text-xs text-muted-foreground">{t('bulkUploadNaming')}</p>}

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
            {isDesktopApp()
              ? <Button type="button" variant="outline" className="w-full justify-start" onClick={() => void chooseFile()}>{files[0]?.name ?? t('fieldFile')}</Button>
              : <Input id="project-file" type="file" multiple accept={reportExtensions.map((extension) => `.${extension}`).join(',')} onChange={(e) => setFiles(Array.from(e.target.files ?? []))} required />}
            {isDesktopApp() && files.length === 0 && <p className="text-xs text-muted-foreground">{t('chooseFileError')}</p>}
            {!isDesktopApp() && <p className="text-xs text-muted-foreground">{t('bulkUploadHint')}</p>}
            {bulk && <p className="text-xs text-muted-foreground">{t('bulkUploadSelected', { count: String(files.length) })}</p>}
          </div>

          {error && <p className="text-destructive text-sm">{error}</p>}

          <DialogFooter>
            <Button type="submit" disabled={submitting || files.length === 0}>
              {progress ? t('bulkUploadProgress', { done: String(progress.done), total: String(progress.total) }) : submitting ? t('uploading') : bulk ? t('bulkUploadAction', { count: String(files.length) }) : t('upload')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
