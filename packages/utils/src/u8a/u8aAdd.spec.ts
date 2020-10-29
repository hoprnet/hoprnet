import { u8aAdd } from './u8aAdd'
import { u8aEquals } from './equals'

import assert from 'assert'

describe('test u8a addition', function () {
  it('should add two u8a with modulo', function () {
    let A = new Uint8Array([1])
    let B = new Uint8Array([1])

    let expected = new Uint8Array([2])
    assert(u8aEquals(u8aAdd(false, A, B), expected), 'add without overflow')
    assert(u8aEquals(A, new Uint8Array([1])))

    A = new Uint8Array([255])
    B = new Uint8Array([1])

    expected = new Uint8Array([0])
    assert(u8aEquals(u8aAdd(false, A, B), expected), 'add with ignored overflow')
    assert(u8aEquals(A, new Uint8Array([255])))

    A = new Uint8Array([0, 255])
    B = new Uint8Array([0, 1])

    expected = new Uint8Array([1, 0])
    assert(u8aEquals(u8aAdd(false, A, B), expected), 'add with overflow')
    assert(u8aEquals(A, new Uint8Array([0, 255])))
  })

  it('should add two u8a with modulo in-place', function () {
    let A = new Uint8Array([1])
    let B = new Uint8Array([1])

    let expected = new Uint8Array([2])
    assert(u8aEquals(u8aAdd(true, A, B), expected), 'add without overflow')
    assert(u8aEquals(A, expected))

    A = new Uint8Array([255])
    B = new Uint8Array([1])

    expected = new Uint8Array([0])
    assert(u8aEquals(u8aAdd(true, A, B), expected), 'add with ignored overflow')
    assert(u8aEquals(A, expected))

    A = new Uint8Array([0, 255])
    B = new Uint8Array([0, 1])

    expected = new Uint8Array([1, 0])
    assert(u8aEquals(u8aAdd(true, A, B), expected), 'add with overflow')
    assert(u8aEquals(A, expected))
  })
})
