import React from "react";
import "../../styles/main.scss";

const BoxDataTable = ({ nodesVerified, nodesRegistered, nodesConnected }) => {
  return (
    <div className="table-esp-data">
      <div>
        <div>
          {nodesVerified && <span>{nodesVerified}</span>}
          <p>Verified</p>
        </div>
      </div>
      <div>
        <div>
          {nodesRegistered && <span>{nodesRegistered}</span>}
          <p>Registered</p>
        </div>
      </div>
      <div>
        <div>
          {nodesConnected && <span>{nodesConnected}</span>}
          <p>Connected</p>
        </div>
      </div>
    </div>
  );
};

export default BoxDataTable;
