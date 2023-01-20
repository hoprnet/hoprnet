import {
  resolve_environment,
  core_misc_set_panic_hook,
  supported_environments,
  type NetworkOptions,
  type ResolvedEnvironment,
  type Environment
} from '../lib/core_misc.js'
core_misc_set_panic_hook()
export {
  resolve_environment,
  supported_environments,
  type NetworkOptions,
  type ResolvedEnvironment,
  type Environment
} from '../lib/core_misc.js'

export type EnvironmentType = 'production' | 'staging' | 'development'

export type ProtocolConfig = {
  environments: {
    [key: string]: Environment
  }
  networks: {
    [key: string]: NetworkOptions
  }
}

const MONO_REPO_PATH = new URL('../../../', import.meta.url).pathname

/**
 * @param version HOPR version
 * @returns environments that the given HOPR version should be able to use
 */
export function supportedEnvironments(): Environment[] {
  return supported_environments(MONO_REPO_PATH)
}

/**
 * @param environment_id environment name
 * @param customProvider
 * @returns the environment details, throws if environment is not supported
 */
export function resolveEnvironment(environment_id: string, customProvider?: string): ResolvedEnvironment {
  return resolve_environment(MONO_REPO_PATH, environment_id, customProvider)
}
