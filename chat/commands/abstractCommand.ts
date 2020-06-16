export default interface Command {
  execute(...props: any[]): void | Promise<void>
  complete(
    line: string,
    cb: (err: Error | undefined, hits: [string[], string]) => void,
    query?: string
  ): void | Promise<void>
}
