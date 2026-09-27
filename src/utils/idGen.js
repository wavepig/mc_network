export const CHARS = "0123456789ABCDEFGHJKLMNPQRSTUVWXYZ";

function group() {
  let out = "";
  for (let i = 0; i < 4; i++) {
    out += CHARS[Math.floor(Math.random() * CHARS.length)];
  }
  return out;
}

export function randomName() {
  return `mc-net-${group()}-${group()}`;
}

export function randomSecret() {
  return `${group()}-${group()}`;
}
