# Bacon

This crate contains the necessary functions to process a finalized block from the Beacon chain as well as being able to process sync committee updates.
By doing so we essentially have a simple version of the Ethereum Light Client protocol. 95% of the code in here is yanked from https://github.com/Snowfork/snowbridge. The main contributions here were to removed all the Substrate related constructs so that we have pure functions that are environment agnostic.