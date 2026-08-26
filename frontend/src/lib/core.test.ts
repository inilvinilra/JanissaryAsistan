import { describe, expect, it } from 'vitest';
import { isPhaseCategory } from './category-groups';
import { categoryLabel, translate } from './i18n';
import { createPdfReport } from './pdf';
import { createXlsxWorkbook } from './xlsx';
import { projectNameFromFile } from './api';

describe('localization', () => {
  it('translates variables and preserves unknown keys', () => {
    expect(translate('en', 'toastProjectAdded', { name: 'Atlas' })).toContain('Atlas');
    expect(translate('tr', 'missing.translation.key')).toBe('missing.translation.key');
    expect(categoryLabel('en', 'ai')).toBe('AI / Machine Learning');
  });
});

describe('category groups', () => {
  it('recognizes only configured phase categories', () => {
    expect(isPhaseCategory('odr')).toBe(true);
    expect(isPhaseCategory('ktr')).toBe(true);
    expect(isPhaseCategory('robotics')).toBe(false);
  });
});

describe('report artifact generators', () => {
  it('creates an XLSX ZIP with escaped worksheet values', async () => {
    const workbook = createXlsxWorkbook([['Name', 'A&B <Project>']]);
    const bytes = new Uint8Array(await workbook.arrayBuffer());
    const content = new TextDecoder().decode(bytes);

    expect(workbook.type).toBe('application/vnd.openxmlformats-officedocument.spreadsheetml.sheet');
    expect(Array.from(bytes.slice(0, 4))).toEqual([0x50, 0x4b, 0x03, 0x04]);
    expect(content).toContain('A&amp;B &lt;Project&gt;');
  });

  it('creates a valid PDF header and escapes PDF control characters', async () => {
    const report = createPdfReport('Jury (Report)', ['Score: 90']);
    const content = await report.text();

    expect(report.type).toBe('application/pdf');
    expect(content.startsWith('%PDF-1.4')).toBe(true);
    expect(content).toContain('Jury \\(Report\\)');
    expect(content).toContain('%%EOF');
  });
});

describe('bulk report import', () => {
  it('derives a readable project name from the report file name', () => {
    expect(projectNameFromFile('akilli-sulama-raporu.pdf')).toBe('akilli sulama raporu');
    expect(projectNameFromFile('Proje_Raporu_v2.docx')).toBe('Proje Raporu v2');
  });

  it('keeps the original name when there is nothing left to strip', () => {
    expect(projectNameFromFile('.gitignore')).toBe('.gitignore');
    expect(projectNameFromFile('rapor')).toBe('rapor');
  });
});
