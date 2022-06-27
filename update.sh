#!/bin/bash

su jief

git pull

. venv/bin/activate

cd medialibrary

maturin develop