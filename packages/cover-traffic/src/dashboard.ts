import blessed from 'blessed'
import contrib from 'blessed-contrib'

function setupDashboard(selfPub: PublicKey) {
  const screen = blessed.screen()
  const grid = new contrib.grid({ rows: 4, cols: 4, screen: screen })
  screen.key(['escape', 'q', 'C-c'], function () {
    return process.exit(0)
  })

  const table = grid.set(0, 0, 3, 2, contrib.table, {
    fg: 'white',
    label: 'Nodes',
    keys: true,
    interactive: true,
    border: { type: 'line', fg: 'cyan' },
    columnSpacing: 2,
    columnWidth: [55, 12, 6, 12] /*in chars*/
  } as any)
  table.focus()

  const inspect = grid.set(0, 2, 2, 2, contrib.table, {
    fg: 'white',
    label: 'Selected',
    keys: false,
    interactive: false,
    border: { type: 'line', fg: 'cyan' },
    columnSpacing: 2, //in chars
    columnWidth: [6, 90] /*in chars*/
  } as any)

  const logs = grid.set(3, 0, 1, 4, contrib.log, { label: 'logs' })

  const ctChan = grid.set(2, 2, 1, 2, contrib.table, {
    label: 'Cover Traffic channels',
    columnWidth: [60, 20]
  })

  table.rows.on('select item', (item) => {
    const id = item.content.split(' ')[0].trim()
    const node = STATE.nodes[id]
    if (node) {
      const data = [
        ['id', node.id.toB58String()],
        ['pubkey', node.pub.toHex()],
        ['addr', node.pub.toAddress().toHex()],
        ['ma', node.multiaddrs.map((x) => x.toString()).join(',')]
      ]
      findChannelsFrom(node.pub).forEach((c, i) => {
        data.push([
          'ch.' + i,
          c.destination.toPeerId().toB58String() + ' ' + c.balance.toFormattedString() + ' - ' + c.status
        ])
      })

      inspect.setData({ headers: ['', ''], data })
    }
  })

  screen.render()

  const update = () => {
    table.setData({
      headers: ['ID', 'Importance', '#Chans', 'Tot.Stk'],
      data: Object.values(STATE.nodes)
        .sort((a: any, b: any) => importance(b.pub).cmp(importance(a.pub)))
        .map((p) => [
          p.id.toB58String(),
          new BigNumber(importance(p.pub).toString()).toPrecision(4, 0),
          findChannelsFrom(p.pub).length,
          new BigNumber(totalChannelBalanceFor(p.pub).toString()).toPrecision(4, 0)
        ])
    })

    var l
    while ((l = STATE.log.pop())) {
      logs.log(l)
    }

    ctChan.setData({
      headers: ['Dest', 'Status'],
      data: STATE.ctChannels.map((p: PublicKey) => {
        const chan = findChannel(selfPub, p)
        let status = 'PENDING'
        if (chan) {
          status = chan.status.toString()
        }
        return [p.toPeerId().toB58String(), status]
      })
    })

    screen.render()
  }
  update()

  return update
}



const update = setupDashboard(selfPub)
main(update)




