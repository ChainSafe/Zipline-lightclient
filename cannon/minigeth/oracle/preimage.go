//go:build !mips
// +build !mips

package oracle

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
)

var preimages = make(map[common.Hash][]byte)
var root = "/tmp/cannon"

func SetRoot(newRoot string) {
	root = newRoot
	err := os.MkdirAll(root, os.ModePerm)
	if err != nil {
		log.Fatal(err)
	}
}

func Preimage(hash common.Hash) []byte {
	val, ok := preimages[hash]
	key := fmt.Sprintf("%s/%s", root, hash)
	// We write the preimage even if its value is nil (will result in an empty file).
	// This can happen if the hash represents a full node that is the child of another full node
	// that collapses due to a key deletion. See fetching-preimages.md for more details.
	err := ioutil.WriteFile(key, val, 0644)
	check(err)
	comphash := crypto.Keccak256Hash(val)
	if ok && hash != comphash {
		panic("corruption in hash " + hash.String())
	}
	return val
}

func Preimages() map[common.Hash][]byte {
	return preimages
}

// PreimageKeyValueWriter wraps the Put method of a backing data store.
type PreimageKeyValueWriter struct{}

// Put inserts the given value into the key-value data store.
func (kw PreimageKeyValueWriter) Put(key []byte, value []byte) error {
	hash := crypto.Keccak256Hash(value)
	if hash != common.BytesToHash(key) {
		panic("bad preimage value write")
	}
	preimages[hash] = common.CopyBytes(value)
	return nil
}

// Delete removes the key from the key-value data store.
func (kw PreimageKeyValueWriter) Delete(key []byte) error {
	return nil
}
