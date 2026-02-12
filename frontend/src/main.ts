// TypeScript source for HearthGlow UI (mirrors compiled app.js)
interface Person {
  id: number;
  first_name: string;
  last_name: string;
  children: Person[];
  star_charts: Array<{ id: number; name: string; description: string; star_count: number; star_total: number; }>;
}

async function createPerson(first: string, last: string, color?: string): Promise<number> {
  const payload: any = { first_name: first, last_name: last };
  if (color) payload.calendar_color = color;
  const res = await fetch('/people', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify(payload) });
  if (!res.ok) throw new Error(await res.text());
  const body = await res.json();
  return body.id as number;
}

interface CreateStarChartPayload {
  name: string;
  description: string;
  person_id: number;
  star_count: number;
  star_total: number;
  color?: string;
}

async function createStarChart(payload: CreateStarChartPayload): Promise<number> {
  const res = await fetch('/stars', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify(payload) });
  if (!res.ok) throw new Error(await res.text());
  const body = await res.json();
  return body.id as number;
}

// ... rest of logic mirrors app.js
