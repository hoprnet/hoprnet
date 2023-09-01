#!/usr/bin/env -S DEBUG=${DEBUG} NODE_OPTIONS=${NODE_OPTIONS} node

// File must be a CommonJS script to make sure it does not include any ESM syntax,
// such as `export` or `import`, which causes incomprensive syntax errors when
// running with old versions of Node.js.

if (!process) {
  throw Error(`Please run with Node.js`)
}

// Gives the process a meaningful name, Node.js default is `node`
process.title = 'hoprd'
Error.stackTraceLimit = 100;

const nodeVersion = process.version

// File is a CommonJS script, so we can safely use `require`
const semver = require('semver')
const pkg = require('../package.json')

if (!pkg.engines || !pkg.engines.node) {
  console.error(`Incorrect package.json file. Please make sure it specifies a minimum Node.js version.`)
}

const minimumVersion = semver.valid(semver.coerce(pkg.engines.node))

if (semver.lt(nodeVersion, minimumVersion)) {
  console.error(
    `Incompatible Node.js version. Got Node.js v${semver.clean(
      nodeVersion
    )} but required is at least Node.js v${semver.clean(minimumVersion)}`
  )
  process.exit(1)
}

;(async function () {
  // Starts the ESM bootstrap process *after* Node.js version has been checked
  await import('./index.js')
})()
