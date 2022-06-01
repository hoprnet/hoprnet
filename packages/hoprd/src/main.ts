#!/usr/bin/env node

if (!process) {
  throw Error(`Please run with Node.js`)
}

// Gives the process a meaningful name, Node.js default is `node`
process.title = 'hoprd'

const nodeVersion = process.version.replace(/v/, '')

const majString = nodeVersion.match(/^[0-9]{1,}/)

let nodeMajVersion: number
if (!majString || majString.length < 1) {
  // Considered to be always incompatible
  nodeMajVersion = -1
} else {
  nodeMajVersion = parseInt(majString[0])
}

if (nodeMajVersion < 12) {
  // Node.js < 12 does not support ES Modules, so we can safely use `require()`
  // to read package.json
  const minimumVersion: string = require('../package.json').engines.node

  throw Error(`Incompatible Node.js version. Please use Node.js ${minimumVersion}`)
}

;(async function () {
  // We are running with Node.js 12+ so we can use ES Modules API
  const fs = await import('fs')
  const semver = await import('semver')

  const pkg = JSON.parse(
    fs
      .readFileSync(
        new URL(
          '../package.json',
          // @ts-ignore
          import.meta.url
        )
      )
      .toString()
  )

  const minimumVersion = semver.valid(semver.coerce(pkg.engines.node))
  if (semver.lt(nodeVersion, minimumVersion)) {
    console.error(
      `Incompatible Node.js version. Got Node.js v${nodeVersion} but required is at least Node.js v${semver.clean(
        minimumVersion
      )}`
    )
    process.exit(1)
  }

  // Start bootstrap process *after* Node.js version has been checked
  await import('./index.js')
})()
