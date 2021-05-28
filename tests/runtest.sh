#!/bin/bash

set -e

TESTDIR="$(dirname "$0")"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -d|--disassemble) DISASSEMBLE=1; ;;
        -f|--file) FILE="$2"; shift ;;
        -r|--rem) REM=1; ;;
        -s|--safe-funcs) SAFE_FUNCS="--safe-function-overrides=$2"; shift ;;
        --stack) STACK="--stack-size=$2"; shift ;;
        --stop-on-string) STOP_ON_STRING="--stop-on-string"; ;;
        -u|--unsafe-funcs) UNSAFE_FUNCS="--unsafe-function-overrides=$2"; shift ;;
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

cargo run --bin glulxtoc -- $FILE --out-dir=$OUTDIR $DISFLAG $DEBUG $SAFE_FUNCS $STACK $STOP_ON_STRING $UNSAFE_FUNCS

if [ "$REM" ]; then
    GLKLIB="$TESTDIR/remglk"
    REMFLAG="-r"
else
    GLKLIB="$TESTDIR/cheapglk"
fi

BUILDDIR="$OUTDIR/$GLKLIB"
mkdir -p $BUILDDIR
export CC=clang
cmake -DGlkLibPath=$GLKLIB -B$BUILDDIR -S$OUTDIR
make -C $BUILDDIR -j$(nproc) --no-print-directory

REGTEST="$TESTDIR/regtest.py"
BIN="$BUILDDIR/$(basename ${FILE%%.*}) -u"
TESTFILE="$FILE.regtest"
echo "Running testfile $TESTFILE"
python $REGTEST -i "$BIN" $TESTFILE $REMFLAG -t 10