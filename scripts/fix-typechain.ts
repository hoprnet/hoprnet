/*
  Fixes typechain issue, see https://github.com/ethereum-ts/TypeChain/issues/193
*/
import { join } from "path";
import { readFile, writeFile } from "fs";

const typesFile = join(
  __dirname,
  "..",
  "types",
  "truffle-contracts",
  "index.d.ts"
);

readFile(typesFile, "utf8", (error, data) => {
  if (error) throw error;

  const result = data
    .replace(/BigNumber/g, "BN")
    .replace('import { BN } from "bignumber.js";', "");

  writeFile(typesFile, result, "utf8", error => {
    if (error) throw error;

    console.log("Successfully patched typechain output!");
  });
});
