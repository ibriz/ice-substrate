#!/bin/bash

./target/release/ice-node -lruntime=debug |& tee >(grep 'INFO' --line-buffered >>info.log) | tee >(grep 'WARN' --line-buffered >>warn.log)
