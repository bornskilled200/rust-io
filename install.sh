#!/bin/bash

cargo build --release
sudo mkdir -p /usr/local/bin
sudo cp ./target/release/pms5003 /usr/local/bin/pms5003

sudo mkdir -p /opt/pms5003
sudo cp pms5003.sh /etc/init.d/pms5003
sudo chmod 755 /etc/init.d/pms5003
sudo update-rc.d pms5003 defaults

# to remove
# sudo update-rc.d -f pms5003 remove