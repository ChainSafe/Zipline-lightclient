const { execSync } = require("node:child_process");
const { deployed } = require("../scripts/lib")

const PRIOR_PERIOD = Number(process.env.PRIOR_PERIOD ?? 1)

async function main() {
  let [c, m, mm] = await deployed()

  const output = execSync(`PRIOR_PERIOD=${PRIOR_PERIOD} node ../shortbarrel/dist/createUpdate.js`, {stdio: "pipe"})

  const finalizedHash = output.slice(64, 96);
  const update = output.slice(96);

  console.log("finalized hash", finalizedHash.toString("hex"))

  console.log(await c.updatePeriod(update, finalizedHash, {gasLimit: 30000000}));
}

main()
