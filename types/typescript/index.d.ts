/*
  Extra types
*/

export type PromiseType<P extends (...args: any) => Promise<any>> = Parameters<Parameters<ReturnType<P>['then']>[0]>[0]
