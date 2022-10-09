const { deploy } = require("../scripts/lib")
const fs = require("fs")
const {execSync} = require('child_process')

async function main() {
  const goldenRoot = execSync("cd mipsevm && go run .", {encoding: "utf8"});

  let [c, m, mm] = await deploy(goldenRoot);
  let json = {
    "Challenge": c.address,
    "MIPS": m.address,
    "MIPSMemory": mm.address,
  }
  console.log("deployed", json)
  try {
    fs.mkdirSync("/tmp/cannon")
  } catch (e) {}
  fs.writeFileSync("/tmp/cannon/deployed.json", JSON.stringify(json))
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
