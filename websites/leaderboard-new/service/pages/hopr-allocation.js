import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import BoxRemember from "../components/micro-components/box-remember";
// import api from "../utils/api";

// const columnsDefaults = [
//   {
//     title: "score",
//     dataIndex: "score",
//     key: "score",
//     className: 'sortBy asc',
//   },
//   {
//     title: "address",
//     dataIndex: "address",
//     key: "address",
//     className: 'sortBy',
//   },
//   {
//     title: "id",
//     dataIndex: "id",
//     key: "id",
//     className: 'sortBy',
//   },
//   {
//     title: "tweetUrl",
//     dataIndex: "tweetUrl",
//     key: "tweetUrl",
//   },
// ];

const dataTable=[
  {
    rank:'1',
    prize:'40,000'
  },
  {
    rank:'2',
    prize:'20,000'
  },
  {
    rank:'3',
    prize:'15,000'
  },
  {
    rank:'4',
    prize:'10,000'
  },
  {
    rank:'5',
    prize:'7,500'
  },
  {
    rank:'6',
    prize:'6,000'
  },
  {
    rank:'7',
    prize:'4,00'
  },
  {
    rank:'8',
    prize:'3,000'
  },
  {
    rank:'9',
    prize:'2,500'
  },
  {
    rank:'10',
    prize:'2,000'
  },
  {
    rank:'11-20',
    prize:'1,500'
  },
  {
    rank:'21-50',
    prize:'1,000'
  },
  {
    rank:'51-100',
    prize:'500'
  },
  {
    rank:'101-200',
    prize:'200'
  },
]

export default function HoprAllocation() {
  // const [data, setData] = useState(undefined);
  // const [columns, setColumns] = useState(columnsDefaults);

  // useEffect(() => {
  //   const fetchData = async () => {
  //     const response = await api.getAllData();
  //     if (response.data) {
  //       const allNodes = response.data.nodes.sort((a, b) => b.score - a.score),
  //         nodes = allNodes.slice(0, allNodes.length > 6 ? 5 : allNodes.length);

  //       setData(nodes);
  //     };
  //   };

  //   fetchData();
  // }, []);

  // const getIntBase = key => {
  //   switch(key) {
  //     case 'address':
  //       return 16;
  //     case 'id':
  //       return 36;
  //     default:
  //       return 10;
  //   }
  // };

  // const onClickSort = key => {
  //   let sSort = '',
  //     aColumns = [...columns];

  //   aColumns.map(item => {
  //     if (item.key === key) {
  //       sSort = item.className.replace('sortBy', '').trim();
  //       sSort = sSort === 'asc' ? 'desc' : 'asc';
  //     }

  //     if (item.className !== undefined) {
  //       item.className = 'sortBy'
  //     }
  //   });
  //   aColumns.find(item => item.key === key).className = 'sortBy ' + sSort;

  //   let aNew = [...data];
  //   aNew = aNew.sort((a, b) => {
  //     let iBase = getIntBase(key),
  //       convertA = parseInt(a[key], iBase),
  //       convertB = parseInt(b[key], iBase)

  //     if (sSort === 'asc') {
  //       return convertA - convertB;
  //     } else {
  //       return convertB - convertA;
  //     }
  //   });

  //   setData(aNew);
  //   setColumns(aColumns);
  // };

  return (
    <Layout>
      <div className="box special-table-top">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Hopr Allocation</h1>
            </div>
            <div className="box-btn">
              <button>
                <a  
                  className="link-top-blog"
                  target="_blank"
                  href='https://medium.com/hoprnet/hopr-bas%C3%B2dino-a-better-bigger-braver-testnet-97f68e1c9b7e'
                  rel="noopener noreferrer">
                  <img src="/assets/icons/medium.svg" alt="Hopr medium" />
                  learn more 
                </a>
              </button>
            </div>
          </div>
        </div>
        <div className="box-main-area remove-all-padding aux-add-top ">
          <div className="box-container-table">
            <table>
              <thead>
                <tr>
                  <th scope="col">
                    Rank
                  </th>
                  <th scope="col">
                    Prize
                  </th>
                </tr>
              </thead>
              <tbody>
                 {dataTable.map((e)=>{
                   const { rank, prize } = e;
                   return(
                     <tr key={rank}>
                       <td data-label="rank">
                          {rank}
                       </td>
                       <td data-label="prize">
                          {prize}
                       </td>
                     </tr>
                   )
                 })}
              </tbody>
            </table>
            {/*  */}
          </div>
          <BoxRemember/>
        </div>
      </div>
    </Layout>
  );
}
