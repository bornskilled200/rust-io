#!/bin/bash

cargo build --release
sudo mkdir -p /usr/local/bin
sudo service pms5003 stop
sudo cp ./target/release/pms5003 /usr/local/bin/pms5003

sudo mkdir -p /opt/pms5003
sudo cp -r public/ /opt/pms5003/public/

sudo cp ./pms5003.service /etc/systemd/system/pms5003.service
sudo systemctl enable pms5003.service
sudo systemctl start pms5003.service