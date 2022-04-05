#!/bin/bash

./target/release/ice-node -lruntime=debug |& tee >(grep 'offchain-worker' --line-buffered >>ocw.log) | tee >(grep '[Airdrop pallet]' --line-buffered >>airdrop.log)
