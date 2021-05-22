#!/bin/sh

set -e

cd "$(dirname "$0")"

./runtest.sh -f glulxercise.ulx -d
./runtest.sh -f glulxercise.ulx -u 27057
./runtest.sh -f advent.ulx