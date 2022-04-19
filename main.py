import asyncio
import json
import os
import shutil

import toml
from alphabet_detector import AlphabetDetector

config = toml.load("./config.toml")

from unicom import Server, UnicomException
import medialibrary
from medialibrary import Library, Tmdb

medialibrary.tmdb_init(config["tmdb"]["key"], config["tmdb"]["language"])

loop = asyncio.get_event_loop()

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


class Executor:
    """In most cases, you can just use the 'execute' instance as a
    function, i.e. y = await execute(f, a, b, k=c) => run f(a, b, k=c) in
    the executor, assign result to y. The defaults can be changed, though,
    with your own instantiation of Executor, i.e. execute =
    Executor(nthreads=4)"""
    def __init__(self, loop=loop, nthreads=1):
        from concurrent.futures import ThreadPoolExecutor
        self._ex = ThreadPoolExecutor(nthreads)
        self._loop = loop

    def __call__(self, f, *args, **kw):
        from functools import partial
        return asyncio.get_running_loop().run_in_executor(self._ex, partial(f, *args, **kw))


execute = Executor()


def movie_maker(user, lib, video_path, movie_id):

    for v in lib.videos(user).path(video_path).results():
        v.full().delete()
    print(f"new movie {video_path}")
    video = lib.new_video(user, video_path, 0)
    video.set_movie(movie_id)
    movie = lib.movie(user, movie_id)
    name = f"{get_normalized_title(movie)}.{movie.release_date[:4]}{os.path.splitext(video.path)[1]}"
    path = get_best_path(config["path"]["movie"], video.size)
    if path is None:
        raise Exception("no space left")

    for v in lib.videos(user).path(os.path.join(path, name)).results():
        v.full().delete()
    print(f"copy movie {name}")
    shutil.copy(video.path, os.path.join(path, name))
    video.set_path(os.path.join(path, name))


def tv_maker(user, lib, video_path, tv_id, season, episode):
    for v in lib.videos(user).path(video_path).results():
        v.full().delete()
    video = lib.new_video(user, video_path, 1)
    video.set_tv(tv_id, season, episode)
    tv = lib.tv(user, tv_id)
    name = f"{get_normalized_title(tv)}/{get_normalized_title(tv)}.s{int(season):02d}e{int(episode):02d}{os.path.splitext(video.path)[1]}"
    path = get_best_path(config["path"]["tv"], video.size)
    if path is None:
        raise Exception("no space left")

    for v in lib.videos(user).path(os.path.join(path, name)).results():
        v.full().delete()

    ensure_dir(os.path.join(path, f"{get_normalized_title(tv)}/"))
    shutil.copy(video.path, os.path.join(path, name))
    video.set_path(os.path.join(path, name))



class TvEssentialHandler:
    @staticmethod
    async def GET(server, user: str):
        recent = server.library.tvs(user).order_by("MAX(Episodes.release_date) DESC").json_results(30)
        last = server.library.tvs(user).order_by("MAX(Videos.adding) DESC").json_results(30)
        return f'{{ "last":{last}, "recent":{recent}}}'.encode()


class MovieEssentialHandler:
    @staticmethod
    async def GET(server, user: str):
        recent = server.library.movies(user).order_by("Movies.release_date DESC").json_results(30)
        last = server.library.movies(user).order_by("MAX(Videos.adding) DESC").json_results(30)
        return f'{{ "last":{last}, "recent":{recent}}}'.encode()


class TvGenreHandler:
    @staticmethod
    async def GET(server):
        return server.library.genre_tv_json().encode()


class MovieGenreHandler:
    @staticmethod
    async def GET(server):
        return server.library.genre_movie_json().encode()


class IndexHandler:
    @staticmethod
    async def GET():
        return b"{}"


class MovieSearchHandler:
    @staticmethod
    async def GET(server, query: str):
        return Tmdb.search_movie_json(query).encode()

class TvSearchHandler:
    @staticmethod
    async def GET(server, query: str):
        return Tmdb.search_tv_json(query).encode()

class PersonHandler:
    @staticmethod
    async def GET(server, url, user: str):
        if len(url[1]) > 0:
            person = server.library.person(user, int(url[1]))
            person.set_movie()
            person.set_tv()
            return person.json().encode()
        else:
            return server.library.persons(user).json_results().encode()

class MovieHandler:
    @staticmethod
    async def GET(server, url, user: str):
        if len(url[1]) > 0:
            movie = server.library.movie(user, int(url[1]))
            movie.set_videos()
            movie.set_persons()
            movie.set_trailers()
            return movie.json().encode()
        else:
            return server.library.movies(user).json_results().encode()

    @staticmethod
    async def PUT(server, url, input_data: "Json", user: str):
        if len(url[1]) > 0:
            if "watched" in input_data:
                server.library.movie(user, int(url[1])).set_watched(input_data["watched"])

class TvHandler:
    @staticmethod
    async def GET(server, url, user: str, episode_id: int = None):
        if len(url) >= 4 and len(url[1]) > 0 and len(url[2]) > 0 and len(url[3]) > 0:
            episode = server.library.tv_episode(user,  int(url[1]), int(url[2]),  int(url[3]))
            if episode is None:
                raise UnicomException("NotFound", "episode not found")
            episode.set_tv()
            episode.set_season()
            episode.set_videos()
            return episode.json().encode()

        elif len(url) >= 3 and len(url[1]) > 0 and len(url[2]) > 0:
            season = server.library.tv_season(user, int(url[1]), int(url[2]))
            season.set_episodes()
            # season.set_episode_videos()
            season.set_tv()
            return season.json().encode()

        elif len(url) >= 2 and len(url[1]) > 0:
            tv = server.library.tv(user, int(url[1]))
            tv.set_seasons()
            tv.set_persons()
            tv.set_trailers()
            return tv.json().encode()

        else:
            if episode_id is not None:
                return server.library.tv_episodes(user).id(episode_id).json_results().encode()
            return server.library.tvs(user).json_results().encode()

    @staticmethod
    async def PUT(server, url, input_data: "Json", user: str):
        if len(url[1]) > 0:
            if "watched" in input_data:
                if len(url) >= 4 and len(url[1]) > 0 and len(url[2]) > 0 and len(url[3]) > 0:
                    server.library.tv_episode(user, int(url[1]), int(url[2]), int(url[3])).set_watched(input_data["watched"])
                elif len(url) >= 3 and len(url[1]) > 0 and len(url[2]) > 0:
                    server.library.tv_season(user, int(url[1]), int(url[2])).set_watched(input_data["watched"])
                elif len(url) >= 2 and len(url[1]) > 0:
                    server.library.tv(user, int(url[1])).set_watched(input_data["watched"])


class StreamHandler:
    @staticmethod
    async def GET(server, url):
        return server.library.video("reader", int(url[1])).path.encode()


class CollectionHandler:
    @staticmethod
    async def GET(server, url, user: str, restrict: int = 0):
        if len(url[1]) > 0:
            collection = server.library.collection(user, int(url[1]))
            collection.set_movie()
            collection.set_tv()
            return collection.json().encode()
        else:
            collections = server.library.collections(user)
            if restrict:
                collections.restrict()

            return collections.json_results().encode()

    @staticmethod
    async def POST(server, input_data: "Json", user: str):
        collection = server.library.new_collection(user, input_data['name'])
        collection.edit_description(input_data['description'])
        collection.save()
        return b"{}"

    @staticmethod
    async def PUT(server, url, input_data: "Json", user: str):
        if len(url[1]) == 0:
            raise Exception("Error collection not found")
        collection = server.library.collection(user, int(url[1]))
        if "movie_id" in input_data:
            print(f"add movie{input_data['movie_id']}")
            collection.add_movie(input_data["movie_id"])
        if "tv_id" in input_data:
            collection.add_tv(input_data["tv_id"])
        if "description" in input_data:
            collection.edit_description(input_data["description"])
        if "poster_path" in input_data:
            collection.edit_poster_path(input_data["poster_path"])
        collection.save()
        return b"{}"


class VideoHandler:
    @staticmethod
    async def GET(server, url, user: str, media_id: int=None, media_type: str=None):
        if len(url[1]) > 0:
            video = server.library.video(user, int(url[1]))
            return video.json().encode()
        else:
            search = server.library.videos(user)
            if media_id:
                search.media_id(media_id)
            if media_type == "movie":
                search.movie()
            if media_type == "tv":
                search.tv()
            return search.json_results().encode()

    @staticmethod
    async def POST(server, input_data: "Json"):
        user = "maker"
        if "movieID" in input_data:
            await execute(movie_maker, user, server.library, input_data['path'], input_data['movieID'])
        elif "tvID" in input_data:
            await execute(tv_maker, user, server.library, input_data['path'], input_data['tvID'],
                          input_data['season'], input_data['episode'])
        else:
            raise Exception(f"media info invalid {input_data}")
        return b"{'created':'ok'}"

    @staticmethod
    async def PUT(server, url, input_data: "Json", user: str):

        if len(url[1]) == 0:
            raise Exception("Error no video id")

        video = server.library.video(user, int(url[1]))

        if video is None:
            raise Exception("Error video not found")

        if "movie_id" in input_data:
            video.set_movie(input_data["movie_id"])

        elif "tv_id" in input_data:
            video.set_tv(input_data["tv_id"],
                         input_data["season_number"],
                         input_data["episode_number"])

        elif "watch_time" in input_data:
            video.set_watch_time(input_data["watch_time"])

        return b"{}"

    @staticmethod
    async def DELETE(server, url, user: str):
        if user != "root" and user != "jief":
            raise Exception("Not allowed")
        if len(url[1]) > 0:
            video = server.library.video(user, int(url[1]))
            if video is None:
                raise Exception("Error video not found")
            video.delete()
            os.remove(video.path)
        else:
            raise Exception("Error no video id")

        return b"{}"

# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    s = Server(config["stream"], "mediaserver")

    s.library = Library(config["db"], config["rsc"])

    s.add_template("./templates/nav.html", "mediaserver/nav.html")
    s.add_folder_template("./templates/")

    s.add_view(r"^/$", "mediaserver/index.html", IndexHandler)
    s.add_view(r"^/video$", "mediaserver/videos.html")
    s.add_view(r"^/video/(\d+)$", "mediaserver/video.html", VideoHandler)
    s.add_view(r"^/movie$", "mediaserver/movies.html")
    s.add_view(r"^/movie/(\d+)$", "mediaserver/movie.html", MovieHandler)
    s.add_view(r"^/tv$", "mediaserver/tvs.html")
    s.add_view(r"^/tv/(\d+)$", "mediaserver/tv.html", TvHandler)
    s.add_view(r"^/tv/(\d+)/season/(\d+)$", "mediaserver/season.html", TvHandler)
    s.add_view(r"^/tv/(\d+)/season/(\d+)/episode/(\d+)$", "mediaserver/episode.html", TvHandler)
    s.add_view(r"^/person$", "mediaserver/persons.html")
    s.add_view(r"^/person/(\d+)$", "mediaserver/person.html", PersonHandler)
    s.add_view(r"^/collection$", "mediaserver/collections.html")
    s.add_view(r"^/collection/(\d+)$", "mediaserver/collection.html", CollectionHandler)
    s.add_view(r"^/modal/create_collection", "mediaserver/create_collection.html")
    s.add_view(r"^/modal/get_collection", "mediaserver/get_collection.html")
    s.add_view(r"^/modal/tvsearch$", "mediaserver/tvsearch.html")
    s.add_view(r"^/modal/moviesearch$", "mediaserver/moviesearch.html")
    s.add_view(r"^/custom_pagination.html", "mediaserver/custom_pagination.html")

    s.add_api(r"^/api/video(?:/(\d+))?(?:/(.+))?$", VideoHandler)
    s.add_api(r"^/api/movie(?:/(\d+))?$", MovieHandler)
    s.add_api(r"^/api/movie/essential", MovieEssentialHandler)
    s.add_api(r"^/api/movie/genre", MovieGenreHandler)
    s.add_api(r"^/api/tv(?:/(\d+))?(?:/season/(\d+))?(?:/episode/(\d+))?$", TvHandler)
    s.add_api(r"^/api/tv/essential", TvEssentialHandler)
    s.add_api(r"^/api/tv/genre", TvGenreHandler)
    s.add_api(r"^/api/person(?:/(\d+))?$", PersonHandler)
    s.add_api(r"^/api/collection(?:/(\d+))?$", CollectionHandler)
    s.add_api(r"^/api/moviesearch$", MovieSearchHandler)
    s.add_api(r"^/api/tvsearch$", TvSearchHandler)

    s.add_dynamic_file(r"^/stream/(\d+)$", StreamHandler)

    s.add_static_file(r"^/js/(.*\.js)$", "js/")
    s.add_static_file(r"^/rsc/(.*)$", config["rsc"]+"/")
    asyncio.run(s.serve_forever())
