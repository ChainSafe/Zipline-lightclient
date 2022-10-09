import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import { execSync } from "node:child_process";

import { config } from "@lodestar/config/default";
import { Api, getClient } from "@lodestar/api";
import {
  computeSyncPeriodAtSlot,
  getCurrentSlot,
} from "@lodestar/state-transition";
import { altair, ssz } from "@lodestar/types";
import { utils } from "ethers";

/// Params

const API_ENDPOINT = "https://lodestar-mainnet.chainsafe.io";

const INPUT_DIRECTORY = "../preimage-cache";

//const EMULATOR_CMD = "cd ../cannon/mipsevm && go run .";
const EMULATOR_CMD = "cd ../shortcut-rs && cargo run -q --";

///

async function getPreviousSyncPeriod(api: Api): Promise<number> {
  const { data } = await api.beacon.getGenesis();
  return Math.max(
    computeSyncPeriodAtSlot(getCurrentSlot(config, data.genesisTime)) - 1,
    0
  );
}

function getEmulatorInput(update: altair.LightClientUpdate): {
  update: Uint8Array;
  updateHash: string;
} {
  const serialized = ssz.altair.LightClientUpdate.serialize(update);
  const hash = utils.keccak256(serialized);
  return { update: serialized, updateHash: hash };
}

///

async function main(): Promise<void> {
  const api = getClient({ baseUrl: API_ENDPOINT }, { config });
  const previousPeriod = await getPreviousSyncPeriod(api);

  console.error(
    `fetching updates for periods ${previousPeriod} and ${previousPeriod + 1}`
  );

  const { data } = await api.lightclient.getUpdates(previousPeriod, 2);

  console.error(`writing emulator inputs`);

  const inputs = data.map(getEmulatorInput);
  await Promise.all(
    inputs.map((input) =>
      writeFile(join(INPUT_DIRECTORY, input.updateHash), input.update)
    )
  );

  const shellCmdStr = `${EMULATOR_CMD} ${inputs
    .map((input) => input.updateHash)
    .join(" ")}`;
  console.error(`calling emulator`, shellCmdStr);

  const out = execSync(shellCmdStr);

  if (out.length !== 64) {
    console.error(
      `expected emulator output of 64 bytes, got ${out.length} bytes`
    );
    process.exit(1);
  }

  // write out finalized block root and ssz-serialized update
  process.stdout.write(
    Buffer.concat([
      out,
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(data[1].finalizedHeader),
      inputs[1].update,
    ])
  );
}

main();
