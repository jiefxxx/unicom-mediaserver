import medialibrary
from medialibrary import Library, Tmdb
import toml

config = toml.load("./config.toml")

medialibrary.tmdb_init(config["tmdb"]["key"], config["tmdb"]["language"])

lib = Library(config["db"], config["rsc"])

user = "movieMaker"

tv_id_cache = {}
for video in lib.videos(user).tv().unknown().results():
    video = video.full()
    try:
        title, season, episode = video.parse_tv()
        if title in tv_id_cache:
            tv_id = tv_id_cache[title]
        else:
            tv_id = Tmdb.search_tv_id(title)
            tv_id_cache[title] = tv_id
        if tv_id is None:
            print(f"Tv id not found: {title}")
            continue
        video.set_tv(tv_id, season, episode)
        print(f"edit episode {video.path} {tv_id}")
    except Exception as e:
        print(f"Error: {video.path} => {e}")
        # traceback.print_exc()

for video in lib.videos(user).movie().unknown().results():
    video = video.full()
    try:
        title, year = video.parse_movie()
        movie_id = Tmdb.search_movie_id(title, year)
        if movie_id is None:
            print(video.path)
            print(f"regex: {title}, {year}")
            continue

        video.set_movie(movie_id)
        print(f"edit movie {video.path} {movie_id}")

    except Exception as e:
        print(f"Error: {video.path} => {e}")
        # traceback.print_exc()




