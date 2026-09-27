export function formatLinkInfo(name, secret, port) {
  return `${name} ${secret} ${port}`;
}

export function parseLinkInfo(text) {
  const parts = String(text).trim().split(/[\s,/]+/).filter(Boolean);
  if (parts.length < 2) return null;
  const result = { name: parts[0], secret: parts[1] };
  if (parts.length >= 3) {
    const p = Number(parts[2]);
    if (Number.isInteger(p) && p >= 1 && p <= 65535) result.port = p;
  }
  return result;
}

export async function copyText(text) {
  await navigator.clipboard.writeText(text);
}

export async function readText() {
  return await navigator.clipboard.readText();
}
