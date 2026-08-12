import http from 'node:http';

const port = Number(process.env.AI_FIXTURE_PORT ?? 4010);
const expectedToken = process.env.AI_FIXTURE_TOKEN ?? '';

function respond(response, status, payload) {
  response.writeHead(status, { 'Content-Type': 'application/json' });
  response.end(JSON.stringify(payload));
}

const server = http.createServer(async (request, response) => {
  if (request.method !== 'POST' || request.url !== '/score') {
    respond(response, 404, { error: 'Not found' });
    return;
  }
  if (expectedToken && request.headers.authorization !== `Bearer ${expectedToken}`) {
    respond(response, 401, { error: 'Unauthorized' });
    return;
  }
  const chunks = [];
  for await (const chunk of request) chunks.push(chunk);
  try {
    const payload = JSON.parse(Buffer.concat(chunks).toString('utf8'));
    if (!payload.document || !Array.isArray(payload.kpis) || payload.kpis.length === 0) {
      respond(response, 400, { error: 'Document and KPI template are required' });
      return;
    }
    const wordBonus = Math.min(10, Math.floor(Number(payload.document.word_count ?? 0) / 100));
    respond(response, 200, {
      kpi_scores: payload.kpis.map((kpi, index) => ({
        name: kpi.name,
        score: Math.min(100, 70 + wordBonus + index),
      })),
    });
  } catch {
    respond(response, 400, { error: 'Invalid JSON payload' });
  }
});

server.listen(port, '127.0.0.1', () => {
  console.log(`AI contract fixture listening at http://127.0.0.1:${port}/score`);
});
