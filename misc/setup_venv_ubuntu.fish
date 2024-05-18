#!/usr/bin/env fish

sudo apt install python3
sudo apt install python3-pip
sudo apt install python3.12-venv
python3 -m venv .venv
source ./.venv/bin/activate.fish
source ./misc/setup_python.sh
