import { test } from 'node:test'
import assert from 'node:assert/strict';

import { generateReport } from '../index.js'

test('generateReport from native', () => {
  const report = generateReport('../../test_repo/package.json');

  assert(report.cjs.includes('react'));
  assert.equal(report.total, 4);
});

test('generateReport from native with specific packages', () => {
  const report = generateReport('../../test_repo/package.json', ['react']);

  assert(report.cjs.includes('react'));
  assert.equal(report.total, 1);
});

