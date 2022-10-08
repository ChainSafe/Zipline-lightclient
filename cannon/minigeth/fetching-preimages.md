# Fetching Preimages

When run in prefetch mode, minigeth's role is to retrieve all the (hash ->
preimage) mappings that are necessary to run the MIPS version of minigeth.

It does so through the use of a few `PrefetchXXX` functions.

What does minigeth needs preimages for?

1. Traversing Merkle Patricia Trie (MPT) nodes (`trie/database.go`/`node`)
    - via `state_object.go`/`GetState` and `trie/trie.go`/`resolveHash`
2. Getting block headers from their hashes (`fake_blockchain.go`/`GetHeader`)
3. Getting the contract code from its hash (`state/database.go`/`ContractCode` &
   `ContractCodeSize`)

Retrieving preimages for headers and code and straighforward, via the JSON-RPC
calls `eth_getBlockByNumber`, `eth_getBlockByHash` and `eth_getCode` (since when
we have a code hash, we also always have the address of that contract).

Traversing MPT nodes is the major use case. Each MPT node is represented as a
hash of its content, where the children are themselves represented as hash of
their own contents. The Merkle root is simply the root node hash.

Fetching MPT node preimages is not as straightforward as the other cases. Geth
does not expose a JSON-RPC call to do this directly (there is `debug_preimage`,
but it is not available on public endpoints).

Instead, we can use `eth_getProof` call to get preimages for all nodes in a
proof for a specific key. There are two cases:

- If the key is present in the MPT, then the (value) proof contains all MPT
  nodes on the path from the root to the node that holds the value associated to
  the key.
- If the key is not present in the MPT, then we have and absence proof. This
  absence proof contains all MPT nodes on the path from the root to the
  "insertion point" for the value. This insertion point could be :
    - a full node whose slot for the value's key is empty
    - a short node that shares a prefix with the suffix of the value's key (i.e.
      to insert the value, you'd have to break down the short node to insert a
      full node)

> **Vocab note: short/full/value vs extension/branch/leaf nodes.**
>
> The yellow paper defines an MPT in terms of extension, branch and leaf nodes.
>
> - A branch node is a node with multiple children and an optional value.
>    - The branch node must have at least two children, or a children and a value.
> - An extension node contains a (non-empty) key segment and a child node.
> - A leaf node contains a (potentially empty) key segment, and a value.
>
> On the other hand, geth divides things slightly differently:
>
> - A full node is equivalent to the yellow paper's branch node.
> - A short node represents both extension nodes and non-empty-key leaf nodes.
> - A value node is a value on its own.
>   - Because of this, it is not necessary to make the value/child distinction within full nodes.
>
> We'll use the geth terminology in this document.

It is not trivial to convince ourselves that `eth_getProof` can be used to
retrieved all the preimages that we will need. The rest of the document will
present an informal proof that it is indeed the case.

## Identifying the problematic case

Given a MPT, there are three basic operations:

- read a value from a key
- insert a (key, value) binding
- delete the binding for a key

However, for our reasoning, it will be useful to classify the operations as follows:

- *access*: accessing an existing binding (reading, overwriting)
- *insertion*: inserting a binding for a key that doesn't have one yet, or
  attempting to delete a key with no existing binding
    - We lump deletion of non-existing keys with insertion, because they require
      finding the "insertion point" where the binding would have been inserted
      if it existed.
- *deletion*: delete an existing binding for a key

Let's walk each case:

- For *access*, every node we will need to access will either:
    - have been retrieved by a `eth_getProof` call against the block transition pre-state
    - have been created during the block transition (meaning we don't need its preimage)

  This holds irrespective of whether the binding existed in the pre-state or not
  (if it did not, the proof will supply us all nodes on the path to its
  insertion point).

  It also doesn't matter what changes to the pre-state were made prior to the
  access: every node on the path was either created by us, or was in the
  proof against the pre-state.

- For *insertion*, the reasoning is similar: every node on the path to the
  insertion point has either been created during the block transition, or was in
  the proof against the pre-state. Insertion also does not require knowledge of
  any other node preimage (in particular, it does not require knowledge of the
  preimages of the children of the insertion point).

- *Deletion* is the tricky case. Each deletion can induce the following
  structural changes to the MPT:
    1. deletion of a value node
    2. deletion of a short node
    3. collapse of a full node that has only a single child left

  Note that `1` always happens, and that it can possibly lead to `2` or `3`,
  while `2` can also lead to `3`. So the potentialy set of changes is `[1]`,
  `[1, 2]`, `[1, 3]` or `[1, 2, 3]`.

  `3` is the problematic case. In a full node collapse, the full node is
  replaced by a short node, which may be fused with its remaining child and/or
  its parent (if they are also short nodes).

  The problem is that to make this determination, we need to know the type of
  the remaining child of the collapsing full node. **If this node existed in the
  pre-state it is *not* included in the proof!**. This is because the node is
  not on the path to the key to the removed (it's either a sibling of a node on
  the path, or the child of the insertion point).

## Solving the problem

Solving the thorny problem of the missing child node preimage requires a few insights:

- We only actually need to know the preimage if the child is a short node. If
  the child is a full node, we can simply copy its hash.
- We can also use `eth_getProof` against the post-state of the block transition.
- We fetch all the nodes via `eth_getProof` *before* applying tree changes.

As well as a very unintuitive final insight:

- If the child is a short node that is present in the pre-state, then either:
    - We already have its preimage from the `eth_getProof` because of another operation.
    - The post-state must contain a short node whose key segment has a suffix
      that is the key segment of the child.

We can understand this by understanding that this short node can undergo the
following transformations:

1. removal
2. split via key insertion
3. extension via the collapse of its parent (if it's a full node)
4. extension via the collapse of its child (if it's a full node)

For cases 1, 2, and 4, this would mean that the pre-state proof for that
operation would have included the node.

In case 3, the MPT state immediately after the operation will include a "child
extension" (short node whose key segment has a suffix that is the key segment of
the child).

At that stage, we can recursively apply the same logic to further operations
that modify the extension. The only difference is that now, in cases 1, 2 and 4,
we need to distinguish between two cases:

- Removal of the short node that leaves an extension of the original child (or
  the child itself) in the state after the operation.
- Operations that don't, which entail that the `eth_getProof` for the key
  touched by the operation would have fetched the child.

If we wanted to prove this formally, we would start from the assumption that the
child has not been fetched by `eth_getProof`, then make an inductive proof over
the list of tree operations starting from the original deletion that causes a
full node collapse, and, assuming that the child is a short node, show that after
each operation, either:

- the child would have been fetched by `eth_getProof`, leading to a contradiction
- an extension of the child (or the child itself) exists in the state after the operation

So, to reiterate, the property this proves is:

> If the child is a short node that is present in the pre-state, then either:
>    - we already have its preimage from the `eth_getProof` because of another operation
>    - the post-state must contain a short node whose key segment has a suffix that is the key segment of the child

Knowing this, for affected deletions, we can start calling `eth_getProof`
against the post-state. Then we post-process every fetched short nodes to derive
every possible "suffix" of those nodes (i.e. short nodes whose key segment is a
suffix of the key segment of the original node), and record a (hash -> preimage)
mapping. One of these suffix could be a child we couldn't get from the pre-state
proof.

(Unfortunately, we can't be more precise than this — we have to post-process
every single short node.)

If we do this, the once it comes time to actually perform the full node
collapse, we have two possible scenarios:

1. The child preimage is known, either because:
    - It was fetched by the pre-state `eth_getProof` for another operation.
    - It was derived by the post-processing of the post-state `eth_getProof`.
2. The child preimage is not known, which means the child *must* be a full node.

Since we don't need to know the preimage if the child is a full node (we can
just copy the hash, and that lets us compute the Merkle root), this solves our
problem! This also means the code must need to handle the path where a preimage
is not found without failing.

## Implementation Note

We have two ways to operationalize the above logic:

1. Call `eth_getProof` against the post-state for every deletion in a block
   transition (this is what we currently do).
2. Attempt deletions without calling `eth_getProof`, record failing deletions
   and re-run them after calling `eth_getProof`.

The first approach is more computationally efficient, which makes the MIPS
challenge trace a bit shorter. The second approach spares useless JSON-RPC
calls, which could be useful if we run against rate limiting.
