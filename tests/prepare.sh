#!/bin/sh

set -e

cd "$(dirname "$0")"

echo "Downloading regtest.py"
curl -s https://raw.githubusercontent.com/erkyrath/plotex/master/regtest.py -o regtest.py

echo "Downloading Cheapglk"
rm -rf cheapglk
curl -Ls https://github.com/erkyrath/cheapglk/archive/refs/heads/master.tar.gz | tar xz
mv cheapglk-master cheapglk -f
echo "Compiling Cheapglk"
cd cheapglk && make && cd ..

echo "Downloading Remglk"
rm -rf remglk
curl -Ls https://github.com/erkyrath/remglk/archive/refs/heads/master.tar.gz | tar xz
mv remglk-master remglk -f
echo "Compiling Remglk"
cd remglk && make && cd ..