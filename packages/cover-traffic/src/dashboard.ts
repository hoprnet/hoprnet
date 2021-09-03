import blessed from 'blessed'
import contrib from 'blessed-contrib'
import type { State, OpenChannels } from './state'
import { findChannel, totalChannelBalanceFor, findChannelsFrom, importance } from './utils'
import { main } from '.'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import { PublicKey } from '@hoprnet/hopr-utils'
import { BigNumber } from 'bignumber.js'

function setupDashboard(selfPub: PublicKey) {
  const screen = blessed.screen()
  const grid = new contrib.grid({ rows: 4, cols: 4, screen: screen })
  let selectedNode: string = undefined
  let _lastState: State

  screen.key(['escape', 'q', 'C-c'], function () {
    return process.exit(0)
  })

  const table = grid.set(0, 0, 3, 2, contrib.table, {
    label: 'Nodes',
    keys: true,
    interactive: true,
    border: { type: 'line', fg: 'cyan' },
    columnSpacing: 2,
    columnWidth: [55, 12, 6, 12] /*in chars*/
  } as any)
  table.focus()

  const inspect = grid.set(0, 2, 2, 2, contrib.table, {
    label: 'Selected',
    keys: false,
    interactive: false,
    border: { type: 'line', fg: 'cyan' },
    columnSpacing: 2, //in chars
    columnWidth: [6, 90] /*in chars*/
  } as any)

  const logs = grid.set(3, 0, 1, 3, contrib.log, { label: 'logs' })
  const stats = grid.set(3, 3, 1, 1, contrib.table, {
    label: 'stats',
    keys: false,
    interactive: false,
    columnSpacing: 2, //in chars
    columnWidth: [10, 20]
  })

  const ctChan = grid.set(2, 2, 1, 2, contrib.table, {
    label: 'Cover Traffic channels',
    keys: false,
    interactive: false,
    columnSpacing: 2, //in chars
    columnWidth: [55, 6, 6, 6, 6, 15]
  })

  table.rows.on('select item', (item) => {
    selectedNode = item.content.split(' ')[0].trim()
    if (selectedNode && _lastState) {
      const node = _lastState.nodes[selectedNode]
      if (node) {
        const data = [
          ['id', node.id.toB58String()],
          ['pubkey', node.pub.toHex()],
          ['addr', node.pub.toAddress().toHex()],
          ['ma', node.multiaddrs.map((x) => x.toString()).join(',')]
        ]
        findChannelsFrom(node.pub, _lastState).forEach((c, i) => {
          data.push([
            'ch.' + i,
            c.destination.toPeerId().toB58String() + ' ' + c.balance.toFormattedString() + ' - ' + c.status
          ])
        })
        inspect.setData({ headers: ['', ''], data })
      }
    }
  })

  screen.render()

  const update = (state: State) => {
    _lastState = state
    table.setData({
      headers: ['ID', 'Importance', '#Chans', 'Tot.Stk'],
      data: Object.values(state.nodes)
        .sort((a: any, b: any) => importance(b.pub, state).cmp(importance(a.pub, state)))
        .map((p) => [
          p.id.toB58String(),
          new BigNumber(importance(p.pub, state).toString()).toPrecision(2, 0),
          findChannelsFrom(p.pub, state).length,
          new BigNumber(totalChannelBalanceFor(p.pub, state).toString()).toPrecision(2, 0)
        ])
    })

    var l
    while ((l = state.log.pop())) {
      logs.log(l)
    }

    ctChan.setData({
      headers: ['Dest', 'Status', '#Sent', '#Fwd', 'Q', 'Balance'],
      data: state.ctChannels.map((c: OpenChannels) => {
        const chan = findChannel(selfPub, c.destination, state)
        let status
        let balance = '-'
        let stats = state.channels[c.destination.toB58String()] || ({} as any)
        if (chan) {
          status = chan.status.toString()
          balance = chan.balance.toFormattedString()
        } else {
          status = 'UNKNOWN'
        }
        return [
          c.destination.toB58String(),
          status,
          stats.sendAttempts || 0,
          stats.forwardAttempts || 0,
          c.latestQualityOf,
          balance
        ]
      })
    })

    stats.setData({ headers: ['', ''], data: [['block', state.block.toString()]] })

    screen.render()
  }
  return update
}

if (require.main === module) {
  const priv = process.argv[2]
  const peerId = privKeyToPeerId(priv)
  const selfPub = PublicKey.fromPeerId(peerId)
  const update = setupDashboard(selfPub)
  main(update, peerId)
}
