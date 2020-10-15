import type { addresses } from '@hoprnet/hopr-ethereum'

export const colors: {
  [key in addresses.Networks]?: string
} = {
  kovan: 'purple',
  xdai: '#48A9A6',
  matic: '#15429d',
  private: 'grey',
}
