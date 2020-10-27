import React from "react";
import "../../styles/main.scss";

const DataUpdateKnow = ({API_LastUpdated}) => {
  
  return (
    <div className="box-info">
      <div>
        <p>
          Last Updated: <span>{API_LastUpdated}</span>
        </p>
      </div>
    </div>
  );
};

export default DataUpdateKnow;
