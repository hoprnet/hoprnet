export { default as Ganache } from './ganache'

let port = 64000 // Use ports in 64XXX range.
export function getNewPort(): number {
  if (port < 65535) {
    return port++
  }
  throw new Error('Out of valid ports')
}
