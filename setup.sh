#!/bin/bash

python3 -m venv venv

. venv/bin/activate

pip install toml
pip install alphabet_detector
pip install pymediainfo
pip install python_magic
pip install maturin

cd medialibrary

maturin develop

