/** Attachments accepted by `POST /projects/{id}/files`. */
export const attachmentExtensions = ['pdf', 'txt', 'md', 'markdown', 'doc', 'docx', 'xls', 'xlsx', 'csv', 'png', 'jpg', 'jpeg', 'webp'];

/**
 * Accepted by `POST /projects/upload`, which parses the file immediately.
 * `doc` is absent on purpose: the parser has no reader for the legacy binary
 * Word format, so offering it would only produce a 415 after the upload.
 */
export const reportExtensions = attachmentExtensions.filter((extension) => extension !== 'doc');

export function isDesktopApp(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function chooseDesktopProjectFile(extensions: string[] = attachmentExtensions): Promise<File | null> {
  if (!isDesktopApp()) return null;
  const [{ open }, { readFile }] = await Promise.all([
    import('@tauri-apps/plugin-dialog'),
    import('@tauri-apps/plugin-fs'),
  ]);
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Project documents', extensions }],
  });
  if (typeof selected !== 'string') return null;
  const data = await readFile(selected);
  const name = selected.split(/[\\/]/).pop() || 'project-file';
  return new File([data], name);
}

export async function saveDesktopFile(blob: Blob, suggestedName: string): Promise<boolean> {
  if (!isDesktopApp()) return false;
  const [{ save }, { writeFile }] = await Promise.all([
    import('@tauri-apps/plugin-dialog'),
    import('@tauri-apps/plugin-fs'),
  ]);
  const target = await save({ defaultPath: suggestedName });
  if (typeof target !== 'string') return false;
  await writeFile(target, new Uint8Array(await blob.arrayBuffer()));
  return true;
}
