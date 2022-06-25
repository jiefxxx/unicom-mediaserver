#!/bin/bash

git pull

. venv/bin/activate

cd medialibrary

maturin develop