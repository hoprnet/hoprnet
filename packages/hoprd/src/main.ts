#!/usr/bin/env node

// Replace default process name (`node`) by `hoprd`
process.title = 'hoprd'

const majorNodeVersion = parseInt(process.version.match(/(?<=^v)[0-9]{1,}/)?.[0] ?? '-1')

if (majorNodeVersion == -1 || majorNodeVersion < 16) {
  throw Error(`Incompatible Node.js version. Please use Node.js 15+`)
}

// Start bootstrap process after Node.js has been checked
import('./index.js')
