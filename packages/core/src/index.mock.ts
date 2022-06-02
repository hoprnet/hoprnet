import type { HoprOptions } from '.'

export const sampleOptions: Partial<HoprOptions> = {
  // TODO: find better sample options
  environment: { id: 'local-testnet', network: { id: 'hardhat' } as any } as any,
  hosts: {
    ip4: {
      ip: '0.0.0.0',
      port: 0
    }
  },
  dataPath: '/tmp/hoprd-data',
  heartbeatInterval: 1000,
  heartbeatVariance: 100,
  heartbeatThreshold: 1000,
  networkQualityThreshold: 0.5,
  onChainConfirmations: 8
}
