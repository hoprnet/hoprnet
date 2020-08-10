import styles from "../styles/Footer.module.css";

export default function Footer() {
  return (
    <footer className={`${styles.container} section`}>
      <a
        href="http://hoprnet.org/"
        className={styles.logo}
        target="_blank"
        rel="noopener noreferrer"
      >
        Powered by <img src="/logo.png" alt="HOPR Logo" />
      </a>
    </footer>
  );
}
