import BlockscoutLink from '../BlockscoutLink'
import { handles } from './handles'
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
      <div className={styles.handles}>
        {handles.map(({ name, icon, url }) => {
          return (
            <div key={name}>
              <a href={url} target="_blank" rel="noopener noreferrer">
                <img src={icon} />
              </a>
            </div>
          )
        })}
      </div>
      <br />
      <br />
      Last Updated: {refreshed}
    </footer>
  )
}

export default Footer
