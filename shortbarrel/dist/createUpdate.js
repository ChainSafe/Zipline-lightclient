var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import { execSync } from "node:child_process";
import { config } from "@lodestar/config/default";
import { getClient } from "@lodestar/api";
import { computeSyncPeriodAtSlot, getCurrentSlot, } from "@lodestar/state-transition";
import { ssz } from "@lodestar/types";
import { utils } from "ethers";
/// Params
var API_ENDPOINT = "https://lodestar-mainnet.chainsafe.io";
var INPUT_DIRECTORY = "../preimage-cache";
var EMULATOR_CMD = "cd ../cannon/mipsevm && go run main.go";
///
function getPreviousSyncPeriod(api) {
    return __awaiter(this, void 0, void 0, function () {
        var data;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, api.beacon.getGenesis()];
                case 1:
                    data = (_a.sent()).data;
                    return [2 /*return*/, Math.max(computeSyncPeriodAtSlot(getCurrentSlot(config, data.genesisTime)) - 1, 0)];
            }
        });
    });
}
function getEmulatorInput(update) {
    var serialized = ssz.altair.LightClientUpdate.serialize(update);
    var hash = utils.keccak256(serialized).slice(2);
    return { update: serialized, updateHash: hash };
}
function shell(cmd) {
    return execSync(cmd, { encoding: "utf8", stdio: "pipe" }).trim();
}
///
function main() {
    return __awaiter(this, void 0, void 0, function () {
        var api, previousPeriod, data, inputs, shellCmdStr;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    api = getClient({ baseUrl: API_ENDPOINT }, { config: config });
                    return [4 /*yield*/, getPreviousSyncPeriod(api)];
                case 1:
                    previousPeriod = _a.sent();
                    console.error("fetching updates for periods ".concat(previousPeriod, " and ").concat(previousPeriod + 1));
                    return [4 /*yield*/, api.lightclient.getUpdates(previousPeriod, 2)];
                case 2:
                    data = (_a.sent()).data;
                    console.error("writing emulator inputs");
                    inputs = data.map(getEmulatorInput);
                    return [4 /*yield*/, Promise.all(inputs.map(function (input) {
                            return writeFile(join(INPUT_DIRECTORY, input.updateHash), input.update);
                        }))];
                case 3:
                    _a.sent();
                    shellCmdStr = "".concat(EMULATOR_CMD, " ").concat(inputs
                        .map(function (input) { return input.updateHash; })
                        .join(" "));
                    console.error("calling emulator", shellCmdStr);
                    //const out = shell(shellCmdStr).split(" ");
                    // write out finalized block root and ssz-serialized update
                    process.stdout.write(Buffer.concat([
                        ssz.phase0.BeaconBlockHeader.hashTreeRoot(data[1].finalizedHeader),
                        inputs[1].update,
                    ]));
                    return [2 /*return*/];
            }
        });
    });
}
main();
