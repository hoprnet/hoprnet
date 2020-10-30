export type Token = number

function getTokens(amount: number): Token[] {
  let result: Token[] = []

  for (let i = amount; i > 0; i--) {
    result.push(i - 1)
  }

  return result
}

export { getTokens }
