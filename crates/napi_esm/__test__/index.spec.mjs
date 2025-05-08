import test from 'ava'

import { generateReportJs } from '../index.js'

test('generateReportJs from native', (t) => {
  const reportStr = generateReportJs('../../test_repo/package.json');
  const report = JSON.parse(reportStr);

  t.pass(
    report.cjs.includes('react'),
  );

  t.is(report.total, 4)
});

test('generateReportJs from native with specific packages', (t) => {
  const reportStr = generateReportJs('../../test_repo/package.json', ['react']);
  const report = JSON.parse(reportStr);

  t.pass(
    report.cjs.includes('react'),
  );

  t.is(report.total, 1)
});

