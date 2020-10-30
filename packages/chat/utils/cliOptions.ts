// allowed CLI options
// @ts-ignore
export const cli_options: string[][] = [
  ['-b', '--bootstrapNode', undefined, 'starts HOPR as a bootstrap node'],
  ['-n', '--network', '<connector>', 'starts HOPR with blockchain connector <connector>'],
  ['-h', '--help', undefined, 'shows this help page'],
  ['-l', '--listConnectors', undefined, 'shows all available connectors'],
  ['-p', '--password', '<password>', 'start HOPR with <password>'],
  ['-v', '--verbose', undefined, 'show debug info'],
  [undefined, '--debug', undefined, 'run HOPR in debug mode [insecure, only used for development]']
].sort((a, b) => {
  let tmpA: string
  let tmpB: string

  if (a[0] === undefined) {
    // @ts-ignore
    tmpA = a[1].slice(2)
  } else {
    tmpA = a[0].slice(1)
  }

  if (b[0] === undefined) {
    // @ts-ignore
    tmpB = b[1].slice(2)
  } else {
    tmpB = b[0].slice(1)
  }

  return tmpA.localeCompare(tmpB, 'en', { sensitivity: 'base' })
})
