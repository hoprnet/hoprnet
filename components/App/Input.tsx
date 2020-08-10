import { useState } from "react";
import styles from "../../styles/App/Input.module.css";
import store from "../../utils/store";

export default function Input(props: { peerId?: string }) {
  const [state, dispatch] = store.useTracked();
  const [peerId, setPeerId] = useState(undefined);
  const [message, setMessage] = useState("");
  const [anonymous, setAnonymous] = useState(!props.peerId);

  return (
    <div className={styles.container}>
      {!props.peerId ? (
        <input
          placeholder="peer id"
          className={styles.input}
          defaultValue={peerId}
          onChange={(e) => setPeerId(e.target.value)}
        ></input>
      ) : undefined}
      <input
        placeholder="message"
        className={styles.input}
        defaultValue={message}
        onChange={(e) => setMessage(e.target.value)}
      ></input>
      <div className={styles.checkbox}>
        anonymous:
        <input
          type="checkbox"
          defaultChecked={anonymous}
          onClick={() => setAnonymous(!anonymous)}
        ></input>
      </div>
      <button
        className="clickable"
        onClick={() => {
          store.methods.sendMessage(
            state,
            dispatch,
            peerId ?? props.peerId,
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
