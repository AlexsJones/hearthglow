// TypeScript source for HearthGlow UI (mirrors compiled app.js)
interface Person {
  first_name: string;
  last_name: string;
  children: Person[];
  star_charts: Array<{ id: number; name: string; description: string; star_count: number; star_total: number; }>;
}

async function createPerson(first: string, last: string): Promise<number> {
  const res = await fetch('/people', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({ first_name: first, last_name: last }) });
  if (!res.ok) throw new Error(await res.text());
  const body = await res.json();
  return body.id as number;
}

// ... rest of logic mirrors app.js
