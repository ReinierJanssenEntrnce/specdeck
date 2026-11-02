// Writes the latest.json that installed apps check for updates (auto-update design decision 4):
// the version, the release page, and per platform the update file's URL in this exact release
// and its signature. Run by the release workflow's publish job.
//
//   node scripts/latest-json.mjs v1.2.1 nickhartjes/specdeck <folder with the .sig files>

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

/** Update files by the platform keys the updater looks for; each has a `.sig` next to it. */
export const PLATFORMS = {
  'darwin-aarch64': 'SpecDeck-arm64.app.tar.gz',
  'darwin-x86_64': 'SpecDeck-x64.app.tar.gz',
  'windows-x86_64': 'SpecDeck-x64-setup.exe',
  'linux-x86_64': 'SpecDeck-x86_64.AppImage',
};

/** The manifest for `tag`; `signature(file)` reads `file.sig`. Fails when one is missing. */
export function latestJson(tag, repo, signature, now = new Date()) {
  const version = tag.replace(/^v/, '');
  const download = `https://github.com/${repo}/releases/download/${tag}`;
  const platforms = Object.fromEntries(
    Object.entries(PLATFORMS).map(([platform, file]) => [platform, { signature: signature(file).trim(), url: `${download}/${file}` }]),
  );
  return {
    version,
    notes: `https://github.com/${repo}/releases/tag/${tag}`,
    pub_date: now.toISOString().replace(/\.\d{3}Z$/, 'Z'),
    platforms,
  };
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [tag, repo, dir] = process.argv.slice(2);
  if (!tag || !repo || !dir) {
    console.error('Usage: node scripts/latest-json.mjs <tag> <owner/repo> <folder with .sig files>');
    process.exit(2);
  }
  const manifest = latestJson(tag, repo, (file) => readFileSync(join(dir, `${file}.sig`), 'utf8'));
  console.log(JSON.stringify(manifest, null, 2));
}
