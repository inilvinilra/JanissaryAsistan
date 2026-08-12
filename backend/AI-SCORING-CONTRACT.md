# AI Scoring Service Contract

Configure the backend with `AI_SCORING_URL`. Every project upload then sends a `POST` request to that URL. If it is not configured, development uses the documented mock scorer.

Optional `AI_SCORING_TOKEN` is sent as a Bearer token.

## Request

```json
{
  "document": {
    "filename": "project-report.pdf",
    "file_type": "Pdf",
    "raw_text": "Extracted project content...",
    "word_count": 850,
    "headings": ["Abstract", "Methodology"],
    "keywords": ["robotics", "vision"],
    "references": ["https://example.org/source"],
    "has_references": true,
    "has_abstract": true,
    "has_conclusion": true,
    "has_methodology": true,
    "language": "English",
    "sections": []
  },
  "kpis": [
    { "name": "Innovation", "weight": 40.0, "description": "Novelty and differentiation" },
    { "name": "Impact", "weight": 60.0, "description": "Expected measurable benefit" }
  ]
}
```

Do not persist or log the request document outside the authorized evaluation workflow.

## Required response

```json
{
  "kpi_scores": [
    { "name": "Innovation", "score": 84.5 },
    { "name": "Impact", "score": 78.0 }
  ]
}
```

The response must contain exactly one score for each requested KPI, using the identical KPI names. Every score must be a finite number from `0` to `100`. Any non-2xx response, timeout, malformed response, duplicate KPI, missing KPI, or invalid score rejects the upload rather than silently using a mock score.

## Extended evaluation lifecycle

The model service or an authorized administrator can persist the explanation-rich evaluation through `PUT /projects/{id}/ai-evaluation`. That payload includes model version, confidence, KPI rationales, strengths, weaknesses, missing information, risks, sources, and similar projects. The backend records an audit event for every update.

## Local contract fixture

Use the fixture only in isolated development environments to verify the HTTP integration before the trained model service is available:

```bash
AI_FIXTURE_TOKEN=local-test-token node backend/scripts/ai-contract-fixture-server.mjs
```

Then configure the backend with `AI_SCORING_URL=http://127.0.0.1:4010/score` and the matching `AI_SCORING_TOKEN`. The fixture validates the request shape and returns a complete deterministic KPI score set. It is not a production scoring model.
