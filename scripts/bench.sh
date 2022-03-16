#!/bin/bash
./target/release/ice-node benchmark \
--chain dev --execution wasm \
--wasm-execution compiled \
--pallet pallet_airdrop \
--extrinsic '*' \
--steps 20 \
--repeat 10 \
--raw=raw.json \
--output ./