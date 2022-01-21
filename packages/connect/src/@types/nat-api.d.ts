declare module 'nat-api' {
  export default class NatAPI {
    /**
     * Determines the public IP using UPNP
     */
    externalIp: (cb: (err: any, ip: string) => void) => void

    _upnpClient: {
      timeout: number
    }

    /**
     * Attempts to map a public port to a private port using UPNP
     */
    map: (
      opts: { privatePort: number; publicPort: number; protocol: 'UDP' | 'TCP' | null; description: string },
      cb: (err: any) => void
    ) => void

    /**
     * Destroys the instance an unmaps all ports
     */
    destroy: (cb: () => void) => void
  }
}
