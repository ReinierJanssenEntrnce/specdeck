// Invite codes as people type them: 12 Crockford base32 characters, shown as 7K2M-9QXH-4TPA.
// Mirrors crates/protocol/src/invite.rs, which has the final say.

const ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
const LENGTH = 12;

/** The code characters in what someone typed or pasted: O→0, I/L→1, anything else dropped. */
export function codeCharacters(input: string): string {
  const fromLink = input.match(/specdeck:\/\/join\/([^\s/]+)/i)?.[1] ?? input;
  let out = '';
  for (const raw of fromLink.toUpperCase()) {
    const c = raw === 'O' ? '0' : raw === 'I' || raw === 'L' ? '1' : raw;
    if (ALPHABET.includes(c)) out += c;
    if (out.length === LENGTH) break;
  }
  return out;
}

/** Formats as the user types: groups of four with dashes. */
export function formatCode(input: string): string {
  return codeCharacters(input).match(/.{1,4}/g)?.join('-') ?? '';
}

export function isCompleteCode(input: string): boolean {
  return codeCharacters(input).length === LENGTH;
}

export function joinLink(code: string): string {
  return `specdeck://join/${formatCode(code)}`;
}
