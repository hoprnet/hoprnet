/*
  Extra types that are missing from `truffle-contracts`
*/

export type Debug = <P extends Promise<any>>(p: P) => Promise<P>;

declare global {
  namespace Truffle {
    const debug: Debug;
  }
}
