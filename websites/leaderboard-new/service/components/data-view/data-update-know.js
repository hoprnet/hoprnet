import React, { useState, useEffect } from "react";
import api from "../../utils/api";
import "../../styles/main.scss";

const DataUpdateKnow = () => {
  // const [API_Channel, SetAPI_Channel] = useState(null);
  // const [API_Coverbot, SetAPI_Coverbot] = useState(null);
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
      {/* <div>
        <p>
          Channel: <span>0x83cA7023c4B1EDB137E1d87B3D05F</span>
        </p>
      </div>
      <div>
        <p>
          Coverbot: <span>0x1d157417E639ACA5581D96236089…</span>
        </p>
      </div> */}
      <div>
        <p>
          Last Updated: <span>{API_LastUpdated}</span>
        </p>
      </div>
    </div>
  );
};

export default DataUpdateKnow;
