import BlockscoutLink from '../BlockscoutLink'
import styles from './Footer.module.css'

interface FooterProps {
  hoprChannelContract: string
  hoprCoverbotAddress: string
  refreshed: string
}

const Footer: React.FC<FooterProps> = ({ hoprChannelContract, hoprCoverbotAddress, refreshed }: FooterProps) => {
  return (
    <footer className={styles.footer}>
      <div>
        <div>
          <BlockscoutLink id={hoprChannelContract}>
            <strong>Channel:</strong>
            {hoprChannelContract && hoprChannelContract.slice(0, 8)}...
          </BlockscoutLink>
        </div>
        <div>
          <BlockscoutLink id={hoprCoverbotAddress}>
            <strong>Coverbot:</strong>
            {hoprCoverbotAddress && hoprCoverbotAddress.slice(0, 8)}...
          </BlockscoutLink>
        </div>
      </div>
      Thanks for helping us create the <a href="https://hoprnet.org/">HOPR</a> network.
      <br />
      <br />
      Last Updated: {refreshed}
      <script src="https://panther.hoprnet.org/script.js" data-site="LCFGMVKB" defer></script>
    </footer>
  )
}

export default Footer
