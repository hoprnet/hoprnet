import { Badge, Box } from '@chakra-ui/core'
import styles from './Header.module.css'
import Logo from '../Logo/Logo'

interface HeaderProps {
  available: string
  locked: string
}

const Header: React.FC<HeaderProps> = ({ available = '', locked = '' }: HeaderProps) => {
  return (
    <header className={styles.header}>
      <Box className={styles.h1}>
        <a href="https://hoprnet.org">
          <Logo />
        </a>
        <h1 className={styles.title}>
          HOPR Incentivized Testnet
          <Badge style={{ marginLeft: '10px' }}>xDAI Network</Badge>
        </h1>
      </Box>
      <Box d="flex" className={styles.stats}>
        <Box>
          <p>
            <strong className="green">{available.length > 1 ? parseFloat(available).toFixed(4) : '0.00'}</strong> HOPR
            Available
          </p>
          <p>
            <strong className="blue">{locked.length > 1 ? parseFloat(locked).toFixed(4) : '0.00'}</strong> HOPR Sent
          </p>
        </Box>
        <Box ml="10px">
          <p>
            <strong className="green">{available.length > 1 ? parseFloat(available).toFixed(4) : '0.00'}</strong> Payout
            Time
          </p>
          <p>
            <strong className="blue">{locked.length > 1 ? parseFloat(locked).toFixed(4) : '0.00'}</strong> Payout Value
          </p>
        </Box>
      </Box>
    </header>
  )
}

export default Header
