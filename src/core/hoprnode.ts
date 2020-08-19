/*
 * This is a very hacky way of letting code outside of nest specify a HOPR node
 * that the server should integrate with.
 *
 * It should be replaced with better code as soon as possible.
 */

import Hopr from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

var NODE: Hopr<HoprCoreConnector>;

export function setNode(node: Hopr<HoprCoreConnector>) {
  if (NODE) {
    throw new Error('Could not set HOPR node - a node already exists')
  }
  NODE = node
}

export function getNode(): Hopr<HoprCoreConnector> | undefined {
  return NODE
}
