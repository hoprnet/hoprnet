import { useState, useEffect } from "react";
import styles from "../../styles/App/Input.module.css";
import store from "../../utils/store";

export default function Input(props: { peerId?: string }) {
  const [state, dispatch] = store.useTracked();
  const [peerId, setPeerId] = useState("");
  const [message, setMessage] = useState("");
  const [anonymous, setAnonymous] = useState(true);

  let hasPeerId = !!props.peerId;
  useEffect(() => {
    hasPeerId = !!props.peerId;
    setAnonymous(!hasPeerId);
  }, [props.peerId]);

  return (
    <div className={`${styles.container} section`}>
      {!props.peerId ? (
        <input
          placeholder="peer id"
          className={styles.input}
          value={peerId}
          onChange={(e) => setPeerId(e.target.value)}
        />
      ) : undefined}
      <input
        placeholder="message"
        className={styles.input}
        value={message}
        onChange={(e) => setMessage(e.target.value)}
      />
      {!hasPeerId && (
        <div className={styles.checkbox}>
          <span>anonymous</span>
          <input
            type="checkbox"
            checked={anonymous}
            onChange={(e) => setAnonymous(e.target.checked)}
          />
        </div>
      )}
      <button
        className="clickable"
        onClick={() => {
          store.methods.sendMessage(
            state,
            dispatch,
            peerId === "" ? props.peerId : peerId,
            message,
            anonymous
          );

          setPeerId("");
          setMessage("");
        }}
      >
        Send
      </button>
    </div>
  );
}
