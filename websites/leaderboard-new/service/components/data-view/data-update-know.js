import React, { useState, useEffect } from "react";
import api from "../../utils/api";
import "../../styles/main.scss";

const DataUpdateKnow = () => {
  const [API_LastUpdated, SetAPI_LastUpdated] = useState(null);

  useEffect(() => {
    const fetchStats = async () => {
      const apiStats = await api.getState();
      if (apiStats.data) {
        let CleanDate = apiStats.data.refreshed.slice(0, -5);
        SetAPI_LastUpdated(CleanDate);
      }
    };
    fetchStats();
  }, []);

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
