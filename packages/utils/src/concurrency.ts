// Create a limiter to resolve a single function that
// returns a promise at a time.
// eg.
//
// let limiter = oneAtATime()
// limiter(() => Promise.resolve('1'))
//
export function oneAtATime() {
  let p = Promise.resolve()
  return function (cb: () => Promise<void>) {
    p = p.then(cb)
    return p
  }
}
