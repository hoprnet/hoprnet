import { bash } from './utils'

export default async () => {
  await bash(`npx truffle run coverage`)
}
