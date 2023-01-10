import semver from 'semver'

// This file contains a wrapper for a JS library that cannot be loaded directly into Rust.

/**
 * Wrapper for semver `semver.coerce`
 *
 * Coerces a string to SemVer if possible
 *
 * @param version
 * @param options
 */
export function coerce_version(version: string | number | semver.SemVer, options?: semver.CoerceOptions): string {
  return semver.coerce(version, options).version
}

/**
 * Wrapper for `semver.satisfies`
 *
 * Return true if the version satisfies the range.
 *
 * @param version
 * @param range
 * @param optionsOrLoose
 */
export function satisfies(
  version: string | semver.SemVer,
  range: string | semver.Range,
  optionsOrLoose?: boolean | semver.RangeOptions
) {
  return semver.satisfies(version, range, optionsOrLoose)
}
