import React from 'react'

interface BlockscoutLinkProps {
  id: string
  children: React.ReactNode
}

const BlockscoutLink: React.FC<BlockscoutLinkProps> = ({ id, children }: BlockscoutLinkProps) => {
  return (
    <a target="_blank" rel="noreferrer" href={'https://blockscout.com/poa/xdai/address/' + id + '/transactions'}>
      {children}
    </a>
  )
}

export default BlockscoutLink
