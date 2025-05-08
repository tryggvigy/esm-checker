const { generateReportJs } = require('../crates/napi_esm');

// Get the report for all dependencies
const report = generateReportJs('./package.json');
console.log('Full report:', JSON.stringify(JSON.parse(report), null, 2));

// Get the report for specific packages
const specificReport = generateReportJs('./package.json', ['react', 'lodash']);
console.log('\nSpecific packages report:', JSON.stringify(JSON.parse(specificReport), null, 2));