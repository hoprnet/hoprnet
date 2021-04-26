/**
 * Infer the return value of a promise
 */
export type PromiseValue<T> = T extends PromiseLike<infer U> ? U : T
