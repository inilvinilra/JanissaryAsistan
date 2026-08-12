const encoder = new TextEncoder();

function escapePdf(value: string): string {
  return value.replace(/[\\()]/g, (character) => `\\${character}`).replace(/[^\x20-\x7e]/g, '?');
}

export function createPdfReport(title: string, lines: string[]): Blob {
  const content = [`BT`, `/F1 18 Tf`, `50 792 Td`, `(${escapePdf(title)}) Tj`, `/F1 10 Tf`];
  for (const line of lines) content.push(`0 -18 Td`, `(${escapePdf(line)}) Tj`);
  content.push(`ET`);
  const stream = content.join('\n');
  const objects = [
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>',
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
    `<< /Length ${encoder.encode(stream).length} >>\nstream\n${stream}\nendstream`,
  ];
  let output = '%PDF-1.4\n';
  const offsets: number[] = [0];
  for (let index = 0; index < objects.length; index += 1) {
    offsets.push(encoder.encode(output).length);
    output += `${index + 1} 0 obj\n${objects[index]}\nendobj\n`;
  }
  const xrefOffset = encoder.encode(output).length;
  output += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n${offsets.slice(1).map((offset) => `${String(offset).padStart(10, '0')} 00000 n \n`).join('')}trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xrefOffset}\n%%EOF`;
  return new Blob([output], { type: 'application/pdf' });
}
