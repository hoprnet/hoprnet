import BN from 'bn.js'

declare namespace Moment {
  const SIZE: number
}
declare interface Moment extends BN {}

export default Moment
