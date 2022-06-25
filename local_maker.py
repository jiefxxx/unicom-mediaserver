import os
import sys
import traceback
import shutil
from alphabet_detector import AlphabetDetector
import medialibrary

import toml

config = toml.load("./library_config.toml")

user = "localMaker"


def get_normalized_title(media):
    if AlphabetDetector().is_latin(media.original_title):
        return media.original_title.replace(" ", ".").lower()
    return media.title.replace(" ", ".").lower()


def check_for_space(p, size):
    stats = shutil.disk_usage(p)
    free_bytes = stats.free
    # print(path, convert_size(free_bytes), convert_size(size))
    if size >= free_bytes:
        return False
    return True


def ensure_dir(p):
    if not os.path.exists(p):
        print("Create ", p)
        os.mkdir(p)


def get_best_path(paths, size):
    for path in paths:
        if check_for_space(path, size):
            return path
    return None


print("start")
medialibrary.tmdb_init(config["tmdb"]["key"], config["tmdb"]["language"])
lib = medialibrary.Library(config["db"], config["rsc"])
print("finish setup")

if len(sys.argv) < 4:
    print("args count failed")
    sys.exit(-2)

if sys.argv[2] == "movie":
    print("begin movie")
    if len(sys.argv) != 4:
        print("args count failed")
        sys.exit(-2)
    video = lib.new_video(user, sys.argv[1], 0)
    print("video created")
    try:
        video.set_movie(int(sys.argv[3]))
        print("video set_movie")
        movie = lib.movie(user, int(sys.argv[3]))
        name = f"{get_normalized_title(movie)}.{movie.release_date[:4]}{os.path.splitext(video.path)[1]}"
        path = get_best_path(config["path"]["movie"], video.size)
        if path is None:
            print("no space left")
            sys.exit(-3)

        for v in lib.videos(user).path(os.path.join(path, name)).results():
            v.full().delete()

        shutil.copy(video.path, os.path.join(path, name))
        print("video copy", name)
        video.set_path(os.path.join(path, name))
        sys.exit(0)
    except Exception as e:
        print(e)
        video.delete()
        sys.exit(-4)


elif sys.argv[2] == "tv":
    if len(sys.argv) != 6:
        print(sys.argv)
        print("args count failed")
        sys.exit(-2)
    video = lib.new_video(user, sys.argv[1], 1)
    try:
        video.set_tv(int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5]))
        tv = lib.tv(user, int(sys.argv[3]))
        name = f"{get_normalized_title(tv)}/{get_normalized_title(tv)}.s{int(sys.argv[4]):02d}e{int(sys.argv[5]):02d}{os.path.splitext(video.path)[1]}"
        path = get_best_path(config["path"]["tv"], video.size)
        if path is None:
            print("no space left")
            sys.exit(-3)

        for v in lib.videos(user).path(os.path.join(path, name)).results():
            v.full().delete()

        ensure_dir(os.path.join(path, f"{get_normalized_title(tv)}/"))
        shutil.copy(video.path, os.path.join(path, name))
        video.set_path(os.path.join(path, name))
        sys.exit(0)
    except Exception:
        print(e)
        video.delete()
        sys.exit(-4)
else:
    print("type unknown")
    sys.exit(-1)









