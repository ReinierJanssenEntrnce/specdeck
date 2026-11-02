// node --test 'scripts/*.test.mjs' (part of just check)
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { PLATFORMS, latestJson } from './latest-json.mjs';

const signatures = (dir) => {
  for (const file of Object.values(PLATFORMS)) writeFileSync(join(dir, `${file}.sig`), `signature of ${file}\n`);
};

test('names each platform’s update file in this release with its signature', () => {
  const manifest = latestJson('v1.2.1', 'nickhartjes/specdeck', (file) => `signature of ${file}\n`, new Date('2026-11-02T12:00:00.123Z'));
  assert.equal(manifest.version, '1.2.1');
  assert.equal(manifest.notes, 'https://github.com/nickhartjes/specdeck/releases/tag/v1.2.1');
  assert.equal(manifest.pub_date, '2026-11-02T12:00:00Z');
  assert.deepEqual(Object.keys(manifest.platforms), ['darwin-aarch64', 'darwin-x86_64', 'windows-x86_64', 'linux-x86_64']);
  assert.deepEqual(manifest.platforms['darwin-aarch64'], {
    signature: 'signature of SpecDeck-arm64.app.tar.gz',
    url: 'https://github.com/nickhartjes/specdeck/releases/download/v1.2.1/SpecDeck-arm64.app.tar.gz',
  });
  assert.equal(manifest.platforms['windows-x86_64'].url.endsWith('/v1.2.1/SpecDeck-x64-setup.exe'), true);
});

test('reads the signatures from a folder, and fails when one is missing', () => {
  const dir = mkdtempSync(join(tmpdir(), 'latest-json-'));
  const run = () => execFileSync('node', [join(import.meta.dirname, 'latest-json.mjs'), 'v1.2.1', 'nickhartjes/specdeck', dir], { stdio: 'pipe' });
  assert.throws(run);
  signatures(dir);
  const manifest = JSON.parse(run().toString());
  assert.equal(manifest.platforms['linux-x86_64'].signature, 'signature of SpecDeck-x86_64.AppImage');
});
