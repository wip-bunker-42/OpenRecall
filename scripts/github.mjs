export const REPOSITORY = 'wip-bunker-42/OpenRecall';
export async function github(path, { method = 'GET', body, allowMissing = false } = {}) {
  const token = process.env.GH_TOKEN;
  if (!token) throw new Error('GH_TOKEN is required for this GitHub operation');
  const response = await fetch(`https://api.github.com/${path}`, {
    method,
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
      ...(body ? { 'Content-Type': 'application/json' } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (response.status === 404 && allowMissing) return null;
  if (!response.ok) throw new Error(`GitHub request failed (${response.status}): ${path}`);
  return response.status === 204 ? null : response.json();
}
export async function pages(path, key) {
  const items = [];
  for (let page = 1; page <= 1000; page++) {
    const result = await github(
      `${path}${path.includes('?') ? '&' : '?'}per_page=100&page=${page}`,
    );
    const batch = key ? result[key] : result;
    if (!Array.isArray(batch)) throw new Error('Unexpected GitHub pagination response');
    items.push(...batch);
    if (batch.length < 100) return items;
  }
  throw new Error('GitHub pagination exceeded its safe limit');
}
export async function notifyOnce({ marker, label, title, body }) {
  const issues = await pages(
    `repos/${REPOSITORY}/issues?state=all&labels=${encodeURIComponent(label)}`,
  );
  const existing = issues.find(
    (issue) =>
      issue.user?.id === 41898282 && issue.user?.type === 'Bot' && issue.body?.includes(marker),
  );
  if (existing) {
    console.log(`Already reported: ${existing.html_url}`);
    return existing;
  }
  const created = await github(`repos/${REPOSITORY}/issues`, {
    method: 'POST',
    body: { title, body: `${marker}\n\n${body}`, labels: [label] },
  });
  console.log(`Created: ${created.html_url}`);
  return created;
}
