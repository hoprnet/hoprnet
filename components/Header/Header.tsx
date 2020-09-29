import styles from "./Header.module.css";
import Logo from "../Logo/Logo";

const Header = ({ available }) => {
  return (
    <header className={styles.header}>
      <div className={styles.h1}>
        <Logo />
        <h1 className={styles.title}>
          <a href="https://hoprnet.org">HOPR</a> Incentivized Testnet on xDAI
        </h1>
      </div>

      <div className={styles.stats}>
        <div>
          <strong className="green">{parseFloat(available).toFixed(4)}</strong>{" "}
          xHOPR Available
        </div>
        {/* <div>
            <strong className="blue">{locked}</strong> xHOPR Locked
          </div> */}
      </div>
    </header>
  );
};

export default Header;
