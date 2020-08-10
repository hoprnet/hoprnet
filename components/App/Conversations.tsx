import styles from "../../styles/App/Conversations.module.css";
import { store } from "../../utils";

export default function Conversations(props: {
  selected: string;
  setSelected: (peerId: string) => void;
}) {
  const [state] = store.useTracked();
  const peerIds = Array.from(state.conversations.keys());

  return (
    <div className={`${styles.container} section`}>
      <div className={styles.list}>
        {peerIds.map((peerId) => {
          const display =
            peerId === "" ? "Anonymous" : `..${peerId.substr(-6)}`;
          const selected = peerId === props.selected;

          return (
            <p
              key={peerId}
              className={`clickable ${selected ? styles.clicked : ""}`}
              onClick={() => props.setSelected(peerId)}
            >
              {display}
            </p>
          );
        })}
      </div>
    </div>
  );
}
