const { execSync } = require("node:child_process");
const { deployed } = require("../scripts/lib")

async function main() {
  let [c, m, mm] = await deployed()

  const output = execSync("node ../shortbarrel/dist/createUpdate.js")

  const finalizedHash = output.slice(0, 32);
  const update = output.slice(32);

  await c.updatePeriod(update, finalizedHash);
}

main()
