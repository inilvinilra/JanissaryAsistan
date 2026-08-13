const allowedExtensions = ['pdf', 'txt', 'md', 'markdown', 'doc', 'docx', 'xls', 'xlsx', 'csv', 'png', 'jpg', 'jpeg', 'webp'];

export function isDesktopApp(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function chooseDesktopProjectFile(): Promise<File | null> {
  if (!isDesktopApp()) return null;
  const [{ open }, { readFile }] = await Promise.all([
    import('@tauri-apps/plugin-dialog'),
    import('@tauri-apps/plugin-fs'),
  ]);
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Project documents', extensions: allowedExtensions }],
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
