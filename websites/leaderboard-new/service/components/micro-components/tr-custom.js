import React from "react";
import "../../styles/main.scss";

const TrCustom = ({ online, address, id, score, tweetUrl }) => {
  return (
    <tr key={id}>
      <td className="icon-help-online" data-label="online">
        <div className="container-online"> 
          <div className={[online ? "online" : "offline"]}></div>
          <p>{online ? 'online' : 'offline'}</p>
        </div>
      </td>
      <td data-label="address" data-raw={address}>
        <a
          className="table-link-on"
          target="_blank"
          href={"https://explorer.matic.network/address/" + address}
          rel="noopener noreferrer"
        >
          <img src="/assets/icons/link.svg" alt="link" />
         <div> {address.slice(0, 5)}<span>...</span>{address.slice(-5)}</div>
        </a>
      </td>
      <td data-label="id" data-raw={id}>
        <div>{id.slice(0, 5)}<span>...</span>{id.slice(-5)}</div>
      </td>
      <td data-type="score" data-label="score">
        {score}
      </td>
      <td data-label="tweetUrl">
        <a target="_blank" href={tweetUrl} rel="noopener noreferrer">
          <img src="/assets/icons/twitter.svg" alt="twitter" />
        </a>
      </td>
    </tr>
  );
};

export default TrCustom;
