import { rmSync } from 'fs'

// Exports for REAL that can be imported by Rust WASM modules

// Wrapper for libraries that do not support named imports
export * from './semver.js'

export function removePathRecursively(path: string) {
    rmSync(path, { recursive: true, force: true })
}