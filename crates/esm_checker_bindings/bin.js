#!/usr/bin/env node

const { generateReport } = require('./index.js')

function printHelp() {
  console.log(`Checks ESM readiness of a project

Usage: checker [OPTIONS] --package-json-location <PACKAGE_JSON_LOCATION>

Options:
  -p, --package-json-location <PACKAGE_JSON_LOCATION>
          package.json file to check
  -o, --outfile <OUTFILE>
          output .json file to write results to (absolute path)
  -c, --check <CHECK>
          The dependencies to check, checks all if omitted
  -h, --help
          Print help
  -V, --version
          Print version`);
}

function parseArgs() {
  const args = process.argv.slice(2);
  const result = {
    packageJsonLocation: null,
    outfile: null,
    check: null
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '-h' || arg === '--help') {
      printHelp();
      process.exit(0);
    } else if (arg === '-V' || arg === '--version') {
      console.log(require('./package.json').version);
      process.exit(0);
    } else if (arg === '-p' || arg === '--package-json-location') {
      result.packageJsonLocation = args[++i];
    } else if (arg === '-o' || arg === '--outfile') {
      result.outfile = args[++i];
    } else if (arg === '-c' || arg === '--check') {
      result.check = args[++i].split(',');
    }
  }

  if (!result.packageJsonLocation) {
    console.error('Error: --package-json-location is required');
    printHelp();
    process.exit(1);
  }

  return result;
}

function formatDuration(startTime) {
  const duration = process.hrtime(startTime);
  const seconds = duration[0];
  const milliseconds = Math.round(duration[1] / 1000000);
  return `${seconds}s ${milliseconds}ms`;
}

function main() {
  const startTime = process.hrtime();
  const args = parseArgs();

  try {
    const report = generateReport(args.packageJsonLocation, args.check);

    if (args.outfile) {
      const fs = require('fs');
      const jsonReport = JSON.stringify(report, null, 2);
      fs.writeFileSync(args.outfile, jsonReport);
      console.log(`Report written to ${args.outfile}`);
    } else {
      console.log('Report:');
      console.log(JSON.stringify(report, null, 2));
    }

    const duration = formatDuration(startTime);
    console.log(`Scanned ${report.total} dependencies`);
    console.log(`ESM: ${report.esm.length}`);
    console.log(`CommonJS: ${report.cjs.length}`);
    console.log(`Faux ESM with CommonJS transitive dependencies: ${report.fauxEsm.withCommonjsDependencies.length}`);
    console.log(`Faux ESM with missing JS file extensions: ${report.fauxEsm.withMissingJsFileExtensions.length}`);
    console.log(`Resolve errors: ${report.resolveErrors.length}`);
    console.log(`Parse errors: ${report.parseErrors.length}`);
    console.log(`Done in ${duration}`);
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

main();
