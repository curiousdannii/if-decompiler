#!/bin/bash

set -e

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -d|--disassemble) DISASSEMBLE=1; ;;
        -f|--file) FILE="$2"; shift ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

if [ "$DISASSEMBLE" ]; then
    OUTDIR="$PWD/$FILE.disassembled"
    DISFLAG="--disassemble"
else
    OUTDIR="$PWD/$FILE.decompiled"
fi

if [ -f "$FILE.gameinfo.dbg" ]; then
    DEBUG="--debug-file=$FILE.gameinfo.dbg"
fi

cargo run --bin glulxtoc -- $FILE --out-dir=$OUTDIR $DISFLAG $DEBUG

CHEAPGLK="$OUTDIR/cheapglk"
mkdir -p $CHEAPGLK
cmake -B$CHEAPGLK -S$OUTDIR
make -C $CHEAPGLK -j$(nproc) --no-print-directory

REGTEST="$(dirname "$0")/regtest.py"
BIN="$CHEAPGLK/$(basename ${FILE%%.*}) -u"
TESTFILE="$FILE.regtest"
echo "Runing testfile $TESTFILE"
python $REGTEST -i "$BIN" $TESTFILE