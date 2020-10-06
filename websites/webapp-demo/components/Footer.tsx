import classNames from 'classnames'
import styles from '../styles/Footer.module.scss'

export default function Footer() {
  return (
    <footer className={classNames(styles.container, 'section')}>
      <a href="http://hoprnet.org/" className={styles.logo} target="_blank" rel="noopener noreferrer">
        Powered by <img src="/logo.png" alt="HOPR Logo" />
      </a>
      <a
        href="https://github.com/hoprnet/hopr-webapp-demo"
        className={classNames(styles.logo, 'img-to-white')}
        target="_blank"
        rel="noopener noreferrer"
      >
        <img src="/icons/github.svg" alt="Github Logo" />
      </a>
    </footer>
  )
}
