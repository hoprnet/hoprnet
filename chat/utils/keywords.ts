// Allowed keywords
export const keywords: string[][] = [
  ['open', 'opens a payment channel'],
  ['send', 'sends a message to another party'],
  ['quit', 'stops the node and terminates the process'],
  ['crawl', 'crawls the network and tries to find other nodes'],
  ['openChannels', 'lists all currently open channels'],
  ['closeAll', 'closes all payment channel of this node'],
  ['myAddress', 'shows the address of this node'],
  ['balance', 'shows our current balance'],
  ['listConnectors', 'lists all installed blockchain connectors'],
  ['ping', 'pings another node to check its availability'],
  ['version', 'shows the versions for `hopr-chat` and `hopr-core`'],
  ['help', 'shows this help page'],
  ['tickets', 'lists tickets of a channel'],
].sort((a, b) => a[0].localeCompare(b[0], 'en', { sensitivity: 'base' }))
