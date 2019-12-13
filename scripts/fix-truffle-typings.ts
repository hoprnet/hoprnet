/*
  Fixes truffle-typings issue, see https://github.com/ethereum-ts/truffle-typings/pull/13#issuecomment-550325019
*/
import { join } from "path";
import { readFile, writeFile } from "fs";

const typesFile = join(
  __dirname,
  "..",
  "node_modules",
  "truffle-typings",
  "index.d.ts"
);

readFile(typesFile, "utf8", (error, data) => {
  if (error) throw error;

  const result = data.replace('import("web3");', 'import("web3").default;');

  writeFile(typesFile, result, "utf8", error => {
    if (error) throw error;

    console.log("Successfully patched truffle-typings library!");
  });
});
