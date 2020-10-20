import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from '../utils/api'


export default function Home() {
  const [data, setData] = useState({})
  const [dataTable, setDataTable] = useState(false)

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        // console.log('All data: ', response.data);
        setData(response.data);
        setDataTable(response.data.connected)
        console.log(response.data.connected)
      }
    }
    fetchData();
  }, []);

  const columns = [
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
      title: "score",
      dataIndex: "score",
      key: "score",
    },
    {
      title: "tweetId",
      dataIndex: "tweetId",
      key: "tweetId",
    },
    {
      title: "tweetUrl",
      dataIndex: "tweetUrl",
      key: "tweetUrl",
    },
  ];


  return (
    <Layout>
      <div className="box">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Leaderboard</h1>
            </div>
            <div className="box-btn">
              <button>
                <img src="/assets/icons/refresh.svg" alt="refresh now" />
                refresh now
              </button>
            </div>
          </div>

          <div className="box-menu-optional">
            <ul>
              <li className="active">All</li>
              <li>Verified</li>
              <li>Registered</li>
              <li>Connected</li>
            </ul>
          </div>
        </div>
        <div className="box-main-area">
          <div className="box-container-table">
            {dataTable && (
            <table id='date'>
            <tbody>
               {/* <tr>.map</tr> */}
               {dataTable.map((e, index)=>{
                 const { address, id, tweetId, tweetUrl } = e;
                return(
                  <tr key={id}>
                  <td>{address}</td>
                   <td>{id}</td>
                   <td>{tweetId}</td>
                   <td><a href={tweetUrl}><img src="/assets/icons/twitter.svg" alt="twitter" /></a></td>
                </tr>
                )
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


