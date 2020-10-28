import fetch from "isomorphic-fetch";
import { HOPR_DATABASE_URL } from "./env";

export const FirebaseNetworkSchema = {
  basodino: 'basodino',
};

export const FirebaseNetworkTables = {
  score: 'score',
  state: 'state'
};

class FirebaseDatabase {
  constructor() {
    this.databaseUrl = `https://${HOPR_DATABASE_URL}.firebaseio.com/`;
  }

  async resolveResponse(response) {
    if (response) {
      const json = await response
        .json()
        .catch((err) =>
          console.error(
            `- resolveResponse | json :: Error parsing data from response`,
            err
          )
        );
      return { data: json, status: 200 };
    } else {
      console.error(`- resolveResponse | Failed to retrieve data.`);
      return { data: null, status: 500 };
    }
  }

  async getSchema(schema) {
    try {
      const response = await fetch(
        `${this.databaseUrl}${schema}.json`
      ).catch((err) =>
        console.error(`- getSchema | fetch :: Error retrieve data from database`, err)
      );
      return this.resolveResponse(response);
    } catch (err) {
      console.error(`- getSchema | catch :: Error retrieving data`, err);
      return { data: null, status: 500 };
    }
  }

  async getTable(schema, table) {
    try {
      const response = await fetch(
        `${this.databaseUrl}${schema}/${table}.json`
      );
      return this.resolveResponse(response);
    } catch (err) {
      console.error(`- getTable | catch :: Error retrieving data`, err);
      return { data: null, status: 500 };
    }
  }
}

export default new FirebaseDatabase();
