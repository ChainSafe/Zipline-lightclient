const { execSync } = require("node:child_process");
const { deployed } = require("../scripts/lib")

async function main() {
  let [c, m, mm] = await deployed()

  const output = execSync("node ../shortbarrel/dist/createUpdate.js", {stdio: "pipe"})

  const finalizedHash = output.slice(64, 96);
  const update = output.slice(96);

  console.log("finalized hash", finalizedHash.toString("hex"))

  console.log(await c.updatePeriod(update, finalizedHash, {gasLimit: 30000000}));
}

main()
