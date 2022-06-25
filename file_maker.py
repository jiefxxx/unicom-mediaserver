import os
import traceback
import magic
from medialibrary import Library

import toml

config = toml.load("./library_config.toml")

user = "fileMaker"

print("start")
lib = Library(config["db"], config["rsc"])
mime = magic.Magic(mime=True)
print("finish setup")


def run_fast_scandir(path):
    ret = []
    for r, d, f in os.walk(path):
        for file in f:
            filename = os.path.join(r, file)
            if mime.from_file(filename).split("/")[0] == "video":
                ret.append(filename)
    return ret


for path_movie in config["path"]["movie"]:
    print(f"enter : {path_movie}")
    movies = run_fast_scandir(path_movie)
    for movie in movies:
        if len(lib.videos(user).path(movie).results()) > 0:
            continue
        try:
            lib.new_video(user, movie, 0)
            print(f"add {movie}")
        except Exception as e:
            print(f"error {movie}")
            traceback.print_exc()

for path_tv in config["path"]["tv"]:
    print(f"enter : {path_tv}")
    tvs = run_fast_scandir(path_tv)
    for tv in tvs:
        if len(lib.videos(user).path(tv).results()) > 0:
            continue
        try:
            lib.new_video(user, tv, 1)
            print(f"add {tv}")
        except Exception as e:
            print(f"error {tv}")
            traceback.print_exc()
