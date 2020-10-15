const gen1 = (async function* gen1() {
  let result = new TextEncoder().encode('A')
  let i = 0

  while (true) {
    await new Promise((resolve) => setTimeout(resolve, 100))
    yield result
    result[0] += 1

    if (i++ == 15) {
      return result
    }
  }
})()

const gen2 = (async function* gen2() {
  let result = 0
  while (true) {
    await new Promise((resolve) => setTimeout(resolve, 150))
    yield result++

    if (result == 23) {
      return result
    }
  }
})()

const it2 = (async function* foo() {
  let aPromise: Promise<IteratorResult<Uint8Array, Uint8Array>> = gen1.next()
  let bPromise: Promise<IteratorResult<number, number>> = gen2.next()

  let aResolved = false
  let bResolved = false

  let aFinished = false
  let bFinished = false

  function aPromiseFunction({ done }: { done?: boolean }) {
    aResolved = true

    if (done) {
      aFinished = true
    }
  }

  function bPromiseFunction({ done }: { done?: boolean }) {
    bResolved = true

    if (done) {
      bFinished = true
    }
  }

  while (true) {
    if (!aFinished && !bFinished) {
      await Promise.race([aPromise.then(aPromiseFunction), bPromise.then(bPromiseFunction)])
    }

    if (aFinished) {
      await bPromise.then(bPromiseFunction)
    }

    if (bFinished) {
      await aPromise.then(aPromiseFunction)
    }

    if (aResolved || bFinished) {
      if (aFinished && bFinished) {
        return (await aPromise).value
      } else {
        yield (await aPromise).value
      }

      aPromise = gen1.next()
      aResolved = false
    }

    if (bResolved || aFinished) {
      if (aFinished && bFinished) {
        return (await bPromise).value
      } else {
        yield (await bPromise).value
      }

      bPromise = gen2.next()
      bResolved = false
    }
  }
})()

async function main() {
  for await (const msg of it2) {
    console.log(`msg`, msg)
  }
}

main()
