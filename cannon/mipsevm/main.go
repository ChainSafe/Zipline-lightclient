package main

import (
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	uc "github.com/unicorn-engine/unicorn/bindings/go/unicorn"
)

func WriteCheckpoint(ram map[uint32](uint32), fn string, step int) {
	trieroot := RamToTrie(ram)
	dat := TrieToJson(trieroot, step)
	fmt.Printf("writing %s len %d with root %s\n", fn, len(dat), trieroot)
	ioutil.WriteFile(fn, dat, 0644)
}

func main() {
	target := -1

	inputHashA, err := hex.DecodeString(strings.TrimPrefix(os.Args[1], "0x"))
	inputHashB, err := hex.DecodeString(strings.TrimPrefix(os.Args[2], "0x"))

	if len(os.Args) > 3 {
		target, _ = strconv.Atoi(os.Args[3])
	}

	if err != nil {
		log.Fatal(err)
	}

	regfault := -1
	regfault_str, regfault_valid := os.LookupEnv("REGFAULT")
	if regfault_valid {
		regfault, _ = strconv.Atoi(regfault_str)
	}

	basedir := "../.."
	root := "../../preimage-cache"

	// step 1, generate the checkpoints every million steps using unicorn
	ram := make(map[uint32](uint32))

	lastStep := 1

	mu := GetHookedUnicorn(root, ram, func(step int, mu uc.Unicorn, ram map[uint32](uint32)) {
		if step == regfault {
			fmt.Printf("regfault at step %d\n", step)
			mu.RegWrite(uc.MIPS_REG_V0, 0xbabababa)
		}
		if step == target {
			SyncRegs(mu, ram)
			fn := fmt.Sprintf("%s/checkpoint_%d.json", root, step)
			WriteCheckpoint(ram, fn, step)
			if step == target {
				// done
				mu.RegWrite(uc.MIPS_REG_PC, 0x5ead0004)
			}
		}
		if step%10000000 == 0 {
			SyncRegs(mu, ram)
			steps_per_sec := float64(step) * 1e9 / float64(time.Now().Sub(ministart).Nanoseconds())
			fmt.Printf("%10d pc: %x steps per s %f ram entries %d\n", step, ram[0xc0000080], steps_per_sec, len(ram))
		}
		lastStep = step + 1
	})

	ZeroRegisters(ram)
	// not ready for golden yet
	LoadMappedFileUnicorn(mu, "../../rust-in-my-cannon/build/rust-in-my-cannon.bin", ram, 0)
	if root == "" {
		WriteCheckpoint(ram, fmt.Sprintf("%s/golden.json", basedir), -1)
		fmt.Println("exiting early without a block number")
		os.Exit(0)
	}

	fmt.Println("Golden root hash %s", RamToTrie(ram))

	// write the inputs into Unicorn memory

	mu.MemWrite(0x30000000, inputHashA[:])
	mu.MemWrite(0x30000020, inputHashB[:])

	fmt.Println("Initial execution root hash %s", RamToTrie(ram))

	mu.Start(0, 0x5ead0004)
	SyncRegs(mu, ram)

	if target == -1 {
		if ram[0x30000800] != 0x1337f00d {
			log.Fatal("failed to output state root, exiting")
		}

		output := []byte{}
		for i := 0; i < 0x44; i += 4 {
			t := make([]byte, 4)
			binary.BigEndian.PutUint32(t, ram[uint32(0x30000800+i)])
			output = append(output, t...)
		}

		fmt.Printf("Output: %x \n", output)

		WriteCheckpoint(ram, fmt.Sprintf("%s/checkpoint_final.json", root), lastStep)

	}

}
