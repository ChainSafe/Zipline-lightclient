package main

import (
	"encoding/hex"
	"encoding/binary"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"time"
	// "bytes"
	"strconv"

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

	inputHashA, err := hex.DecodeString(os.Args[1])
	inputHashB, err := hex.DecodeString(os.Args[2])

	if len(os.Args) > 3 {
		target, _ = strconv.Atoi(os.Args[3])
	}

	// inputHashA, err := hex.DecodeString("e4c2cee3a9455c2b7c0449152a8c7e1a7b811353e4ea2c1dbe1cbe0c790b45f7")
	// inputHashB, err := hex.DecodeString("dead69239826edd5ac0abfe3a69e916e7479ad44e834e35a08e4df7601732a85")

	if err != nil {
		log.Fatal(err)
	}

	// root := ""

	regfault := -1
	regfault_str, regfault_valid := os.LookupEnv("REGFAULT")
	if regfault_valid {
		regfault, _ = strconv.Atoi(regfault_str)
	}

	// basedir := os.Getenv("BASEDIR")
	// if len(basedir) == 0 {
	basedir := "../.."
	root := "../../preimage-cache"
	// }
	// if len(os.Args) > 1 {
	// 	blockNumber, _ := strconv.Atoi(os.Args[1])
	// 	root = fmt.Sprintf("%s/%d_%d", basedir, 0, blockNumber)
	// }
	// if len(os.Args) > 2 {
	// 	target, _ = strconv.Atoi(os.Args[2])
	// }
	evm := false
	// if len(os.Args) > 3 && os.Args[3] == "evm" {
	// 	evm = true
	// }

	// step 1, generate the checkpoints every million steps using unicorn
	ram := make(map[uint32](uint32))

	lastStep := 1
	if evm {
		// TODO: fix this
		/*ZeroRegisters(ram)
		LoadMappedFile("mipigo/minigeth.bin", ram, 0)
		WriteCheckpoint(ram, "/tmp/cannon/golden.json", -1)
		LoadMappedFile(fmt.Sprintf("%s/input", root), ram, 0x30000000)
		RunWithRam(ram, target-1, 0, root, nil)
		lastStep += target - 1
		fn := fmt.Sprintf("%s/checkpoint_%d.json", root, lastStep)
		WriteCheckpoint(ram, fn, lastStep)*/
	} else {
		mu := GetHookedUnicorn(root, ram, func(step int, mu uc.Unicorn, ram map[uint32](uint32)) {
			// it seems this runs before the actual step happens
			// this can be raised to 10,000,000 if the files are too large
			//if (target == -1 && step%10000000 == 0) || step == target {
			// first run checkpointing is disabled for now since is isn't used
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


		// LoadMappedFileUnicorn(mu, fmt.Sprintf("%s/input", root), ram, 0x30000000)

		mu.Start(0, 0x5ead0004)
		SyncRegs(mu, ram)
	}

	if target == -1 {
		if ram[0x30000800] != 0x1337f00d {
			log.Fatal("failed to output state root, exiting")
		}

		// output_filename := fmt.Sprintf("%s/output", root)
		// outputs, err := ioutil.ReadFile(output_filename)
		// check(err)
		// real := append([]byte{0x13, 0x37, 0xf0, 0x0d}, outputs...)

		output := []byte{}
		for i := 0; i < 0x44; i += 4 {
			t := make([]byte, 4)
			binary.BigEndian.PutUint32(t, ram[uint32(0x30000800+i)])
			output = append(output, t...)
		}

		// if bytes.Compare(real, output) != 0 {
		// 	fmt.Println("MISMATCH OUTPUT, OVERWRITING!!!")
		// 	ioutil.WriteFile(output_filename, output[4:], 0644)
		// } else {
		// 	fmt.Println("output match")
		// }
		// 
		fmt.Printf("Output: %x \n", output);

		WriteCheckpoint(ram, fmt.Sprintf("%s/checkpoint_final.json", root), lastStep)

	}

	// step 2 (optional), validate each 1 million chunk in EVM

	// step 3 (super optional) validate each 1 million chunk on chain

	//RunWithRam(ram, steps, debug, nil)

}

// func main() {
// 	// read input hashes from command line
// 	programPath := os.Args[1]

// 	inHashAStr := os.Args[2]
// 	inHashBStr := os.Args[3]

// 	inputHashA, err := hex.DecodeString(inHashAStr)
// 	inputHashB, err := hex.DecodeString(inHashBStr)

// 	if err != nil {
// 		log.Fatal(err)
// 	}

// 	ram := make(map[uint32](uint32))
// 	ZeroRegisters(ram)

// 	lastStep := 1
// 	mu := GetHookedUnicorn("../../preimage-cache", ram, func(step int, mu uc.Unicorn, ram map[uint32](uint32)) {
// 		if step%1000000 == 0 {
// 			steps_per_sec := float64(step) * 1e9 / float64(time.Now().Sub(ministart).Nanoseconds())
// 			fmt.Printf("%10d pc: %x steps per s %f ram entries %d\n", step, ram[0xc0000080], steps_per_sec, len(ram))
// 		}

// 		// we can use this to debug or whatever
// 		lastStep = step + 1
// 	})

// 	LoadMappedFileUnicorn(mu, programPath, ram, 0)
	
// 	// write the inputs into Unicorn memory
// 	mu.MemWrite(0x30000000, inputHashA[:])
// 	mu.MemWrite(0x30000020, inputHashB[:])

// 	// also update our ram trie
// 	LoadData(inputHashA[:], ram, 0x30000000)
// 	LoadData(inputHashB[:], ram, 0x30000020)

// 	// start!
// 	mu.Start(0, 0x5ead0004)
// 	SyncRegs(mu, ram)

// 	if ram[0x30000800] != 0x1337f00d {
// 		log.Fatal("failed to produce result. Exiting")
// 	}

// 	// Read the special output memory
// 	output, _ := mu.MemRead(0x30000804, 0x04)

// 	// print out the final execution snapshot root
// 	if output[0] == 0xff {
// 		// write out how many steps and the terminal snapshot hash
// 		trieroot := RamToTrie(ram)
// 		fmt.Printf("Validation success\n")
// 		fmt.Printf("FinalSystemStateRoot: %s, Stes: %d\n", trieroot, lastStep)
// 	} else {
// 		fmt.Printf("Execution complete but update is invalid\n")
// 	}
// }
