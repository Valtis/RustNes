#!/bin/sh
set -ex
wget https://www.libsdl.org/release/SDL2-2.0.3.tar.gz
tar -xzvf SDL2-2.0.3.tar.gz
cd SDL2-2.0.3/
./configure
make
sudo make install
cd ..
rm -rf SDL2-2.0.3