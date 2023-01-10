// Exports for REAL that can be imported by Rust WASM modules
export * from './io.js'

// Wrapper for libraries that do not support named imports
export * from './semver.js'

// Add your Rust WASM crate exports here
export { dummy_get_one } from '../lib/real_base.js'
