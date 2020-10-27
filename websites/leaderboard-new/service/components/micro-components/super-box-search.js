import React from "react";
import "../../styles/main.scss";

const SuperBoxSearch = ({
  nodesVerified,
  nodesRegistered,
  nodesConnected,
  searchTerm,
  setSearchTerm,
  match,
}) => {
  const handleChange = (event) => {
    setSearchTerm(event.target.value);
  };
  return (
    <div className="aux-super-tab">
      <div className="data-form">
        <p className="help-total-results">
          Total results: <span>{match}</span>
        </p>
        <input
          className="search"
          type="text"
          placeholder="Search by address or ID"
          value={searchTerm}
          onChange={handleChange}
        />
      </div>
      <div className="data-additional">
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
    </div>
  );
};

export default SuperBoxSearch;
