import styles from './Logo.module.css'
import { Image } from '@chakra-ui/core'

const Logo = () => {
  return <Image style={{ margin: '10px auto'}} src="/hopr.png" alt="HOPR Logo" className={styles.logo} />
}

export default Logo
