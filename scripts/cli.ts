import process from 'process'
import { default as build } from './operations/build'
import { default as coverage } from './operations/coverage'
import { default as fund } from './operations/fund'
import { default as migrate } from './operations/migrate'
import { default as network } from './operations/network'
import { default as test } from './operations/test'

const operations = {
  build,
  coverage,
  fund,
  migrate,
  network,
  test
}

const [, , operation, ...rest] = process.argv

if (typeof operation === 'undefined') {
  throw Error('operation not provided')
}

const fn = operations[operation]

if (typeof fn === 'undefined') {
  throw Error(`operation '${operation}' not found`)
}

fn(...rest).catch(console.error)
