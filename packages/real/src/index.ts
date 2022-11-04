// Exports for REAL that can be imported by Rust WASM modules
export * from './io.js'

// Add your Rust WASM crate exports here
export { dummy_get_one } from '../lib/real_base.js'
