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
    .replace('import { BN } from "bignumber.js";', "")
    .replace(/BigNumber/g, "BN");

  writeFile(typesFile, result, "utf8", function(error) {
    if (error) throw error;

    console.log("Successfully patched typechain output!");
  });
});
