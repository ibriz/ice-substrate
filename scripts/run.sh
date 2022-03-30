#!/bin/bash
./target/release/ice-node -lruntime=debug 2>&1 | tee >(grep 'INFO' >> info.log) >(grep 'WARN' >> warn.log) ice.log