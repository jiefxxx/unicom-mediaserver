#!/bin/sh

cd /home/jief/mediaserver/unicom-mediaserver
. ../mediaserver_venv/bin/activate

python3 file_maker.py
python3 movie_maker.py