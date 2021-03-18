export function oneAtATime() {
  let p = Promise.resolve()
  return function (cb: () => Promise<void>) {
    p = p.then(cb)
    return p
  }
}
