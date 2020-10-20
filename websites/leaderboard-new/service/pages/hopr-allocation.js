import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from "../utils/api";

export default function HoprAllocation() {
  const [data, setData] = useState(undefined);
  const allNodes = data ? data.nodes.sort((a, b) => b.score - a.score) : [];

  const nodes = allNodes.slice(0, allNodes.length > 6 ? 5 : allNodes.length);
  
  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      
      if (response.data) setData(response.data);
    };
    fetchData();
  }, []);

  const sfn = (key) =>{
    if (nodes){
      if(nodes.length){
        setData(nodes.sort((a, b) => b[key] - a[key]));
      }
    }
  };

  const columns = [
    {
      title: "score",
      dataIndex: "score",
      key: "score",
    },
    {
      title: "address",
      dataIndex: "address",
      key: "address",
    },
    {
      title: "id",
      dataIndex: "id",
      key: "id",
    },
   
    {
      title: "tweetUrl",
      dataIndex: "tweetUrl",
      key: "tweetUrl",
    },
  ];

  return (
    <Layout>
      <div className="box special-table-top">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Hopr Allocation</h1>
            </div>
          </div>
        </div>
        <div className="box-main-area remove-all-padding aux-add-top ">
          <div className="box-container-table">
            {nodes && (
              <table id="date">
                <thead>
                  <tr>
                    {columns.map((e, index) => {
                      const { title, key } = e;
                      return (
                        <th onClick={() => sfn({key})} scope="col" key={key}>
                          {title}
                        </th>
                      );
                    })}
                  </tr>
                </thead>
                <tbody>
                  {nodes.map((item) => {
                    const { id, address, score,  tweetUrl } = item;
                    return (
                      <tr key={id}>
                         <td data-type="score" data-label="score">
                         <span>
                            <img
                              src="/assets/icons/top.svg"
                              alt="hopr Top ASSETS"
                            />
                          </span>
                          {score}
                        </td>
                        <td data-label="address">{address}</td>
                        <td data-label="id">{id}</td>
                        <td data-label="tweetUrl">
                          <a href={tweetUrl}>
                            <img
                              src="/assets/icons/twitter.svg"
                              alt="twitter"
                            />
                          </a>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            )}
          </div>
        </div>
      </div>
    </Layout>
  );
}
