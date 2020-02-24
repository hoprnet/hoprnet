/*
  extract abi and bytecode into seperate files
*/
import {join} from "path";
import {readdir, readFile, writeFile, mkdir} from "fs";
import {promisify} from "util";

type Output = {
  format: string;
  items: {
    name: string;
    value: any;
  }[];
};

const inputPath = join(__dirname, "..", "build", "contracts");
const outputPath = join(__dirname, "..", "build", "extracted");

const getData = () => {
  return promisify(readdir)(inputPath)
    .then(list => {
      return list
        .filter(file => file.startsWith("Hopr"))
        .map(file => {
          return join(inputPath, file);
        });
    })
    .then(files => {
      return Promise.all(
        files.map(file => {
          return promisify(readFile)(file, {
            encoding: "utf8"
          })
            .then(txt => JSON.parse(txt))
            .then(json => {
              return {
                contractName: json.contractName,
                abi: json.abi,
                bytecode: json.bytecode
              };
            });
        })
      );
    });
};

const writeData = (output: Output) => {
  return Promise.all(
    output.items.map(item => {
      return promisify(writeFile)(
        join(outputPath, `${item.name}${output.format}.json`),
        JSON.stringify(item.value, null, 2)
      );
    })
  );
};

const start = async () => {
  const data = await getData();

  const {abis, bytecodes} = data.reduce<{
    [key: string]: Output;
  }>(
    (result, output) => {
      result.abis.items.push({
        name: output.contractName,
        value: output.abi
      });

      result.bytecodes.items.push({
        name: output.contractName,
        value: output.bytecode
      });

      return result;
    },
    {
      abis: {
        format: "Abi",
        items: []
      },
      bytecodes: {
        format: "Bytecode",
        items: []
      }
    }
  );

  await promisify(mkdir)(outputPath, {
    recursive: true
  });

  return Promise.all([writeData(abis), writeData(bytecodes)]);
};

start().catch(error => {
  console.error(error);
  throw error;
});
