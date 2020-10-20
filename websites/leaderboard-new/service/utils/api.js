import db, { FirebaseNetworkTables } from "./db";
import { HOPR_NETWORK } from "./env";

export async function getData(table) {
  try {
    const queryResponse = await db.getTable(HOPR_NETWORK, table);
    if (queryResponse.data) {
      return { data: queryResponse.data, status: 200 };
    } else {
      return { data: null, status: 500 };
    }
  } catch (e) {
    console.log(e);
    return { data: null, status: 500 };
  }
}

export async function getState() {
  return getData(FirebaseNetworkTables.state);
}

export async function getScore() {
  return getData(FirebaseNetworkTables.score);
}

export async function getAllData() {
  let stats = await getData(FirebaseNetworkTables.state),
    scores = await getData(FirebaseNetworkTables.score),
    sKey = '';

  if (stats) {
    if (stats.data) {
      stats.data.scoreArray = [];
      for (sKey in scores.data) {
        stats.data.scoreArray.push({
          address: sKey,
          score: scores.data[sKey],
        });
      }

      stats.data.connected.map(item => {
        let exist = stats.data.scoreArray.find(single => single.address === item.address);

        if (exist) {
          item.score = exist.score;
        }
      });
    }
  }

  return stats;
}

export default { getState, getScore, getAllData };
