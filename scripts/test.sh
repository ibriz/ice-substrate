#!/bin/bash

SCRIPTDIR=$PWD
for d in $(ls -d ./pallets/*/) ; do
    PALLET_DIR="$SCRIPTDIR/$d"
    cd $PALLET_DIR
    
    TEST_FILE="$PALLET_DIR/./test.sh"
    if [ -f "$TEST_FILE" ]; then
        bash $TEST_FILE
    else
        cargo test
    fi

done

cargo check