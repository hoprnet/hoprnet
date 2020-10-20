import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from "../utils/api";

export default function TopAssets() {
  const [data, setData] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        if (response.data.connected) {
          let connected = response.data.connected.sort((a, b) => b.score - a.score);
          if (connected.length) {
            setData(connected.slice(0, connected.length > 6 ? 5 : connected.length));
          }
        }
      }
    };
    fetchData();
  }, []);

  return (
    <Layout>
      <div className="box">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Top Assets</h1>
            </div>
            <div className="box-btn">
              <button>
                <img src="/assets/icons/refresh.svg" alt="refresh now" />
                refresh now
              </button>
            </div>
          </div>
        </div>
        <div className="box-main-area">
          <div className="box-container-table">
            <div className="box-coming-soon">
              {data && (
                <ul>
                  {data.map(item => {
                    const { id, address, score, tweetId, tweetUrl } = item;
                    return (
                      <li>
                        <address>
                          {id}<br/>
                          {address}<br/>
                          {score}<br/>
                          {tweetId}<br/>
                          {tweetUrl}
                        </address>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
