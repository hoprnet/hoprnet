async function main() {
  const it = (async function* () {
    await new Promise((resolve) => setTimeout(resolve, 100))

    throw Error()
  })()

  // try {
  //     for await (const msg of it) {
  //         console.log(msg)
  //     }
  // } catch (err) {
  //     console.log(`err caught`, err)
  // }
}

main()
