import BN from 'bn.js'

declare namespace Moment {
  const SIZE: number
}
declare interface Moment extends BN {
  new (moment: BN, ...props: any[]): Moment
}

export default Moment
