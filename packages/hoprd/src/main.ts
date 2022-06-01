#!/usr/bin/env node

// Replace default process name (`node`) by `hoprd`
process.title = 'hoprd'

const majorNodeVersion = process.version.match(/(?<=^v)[0-9]{1,}/)

if (!majorNodeVersion || majorNodeVersion.length == 0 || parseInt(majorNodeVersion[0]) < 16) {
  throw Error(`Incompatible Node.js major version. Please use at least Node.js 16`)
}

// Start bootstrap process *after* Node.js version has been checked
import('./index.js')
