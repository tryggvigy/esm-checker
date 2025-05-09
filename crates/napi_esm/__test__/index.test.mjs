import { test } from 'node:test'
import assert from 'node:assert/strict';

import { generateReportJs } from '../index.js'

test('generateReportJs from native', () => {
  const reportStr = generateReportJs('../../test_repo/package.json');
  const report = JSON.parse(reportStr);

  assert(report.cjs.includes('react'));

  assert.equal(report.total, 4)
});

test('generateReportJs from native with specific packages', () => {
  const reportStr = generateReportJs('../../test_repo/package.json', ['react']);
  const report = JSON.parse(reportStr);

  assert(report.cjs.includes('react'));

  assert.equal(report.total, 1)
});

