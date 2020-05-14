import process from 'process'
import { getOperations } from './operations/utils'

const operations = getOperations()
const [, , operation, ...rest] = process.argv

if (typeof operation === 'undefined') {
  throw Error('operation not provided')
}

if (!operations.includes(operation)) {
  throw Error(`operation '${operation}' does not exist`)
}

import(`./operations/${operation}`)
  .then((res) => {
    const fn: (...args: any[]) => Promise<any> = res.default
    if (typeof fn === 'undefined') {
      throw Error(`operation '${operation}' not found`)
    }

    return fn(...rest)
  })
  .catch(console.error)
