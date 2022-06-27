import asyncio
import json
import os
import shutil

import toml

from alphabet_detector import AlphabetDetector

import medialibrary
from medialibrary import Library, Tmdb

library_config = toml.load("./library_config.toml")

async def config(server):
    
    medialibrary.tmdb_init(library_config["tmdb"]["key"], library_config["tmdb"]["language"])

    server.create_user_data("medialib", Library(library_config["db"], library_config["rsc"]))
    server.create_user_data("executor", Executor())

    s = server.config
    s.add_api(r"VideoHandler", VideoHandler)
    s.add_api(r"MovieHandler", MovieHandler)
    s.add_api(r"MovieEssentialHandler", MovieEssentialHandler)
    s.add_api(r"MovieGenreHandler", MovieGenreHandler)
    s.add_api(r"TvHandler", TvHandler)
    s.add_api(r"TvEssentialHandler", TvEssentialHandler)
    s.add_api(r"TvGenreHandler", TvGenreHandler)
    s.add_api(r"PersonHandler", PersonHandler)
    s.add_api(r"CollectionHandler", CollectionHandler)
    s.add_api(r"MovieSearchHandler", MovieSearchHandler)
    s.add_api(r"TvSearchHandler", TvSearchHandler)
    s.add_api(r"StreamHandler", StreamHandler)
    return s

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
    def __init__(self, nthreads=1):
        from concurrent.futures import ThreadPoolExecutor
        self._ex = ThreadPoolExecutor(nthreads)
        self._loop = asyncio.get_event_loop()

    def __call__(self, f, *args, **kw):
        from functools import partial
        return asyncio.get_running_loop().run_in_executor(self._ex, partial(f, *args, **kw))


def movie_maker(user, lib, video_path, movie_id):

    for v in lib.videos(user).path(video_path).results():
        v.full().delete()
    print(f"new movie {video_path}")
    video = lib.new_video(user, video_path, 0)
    video.set_movie(movie_id)
    movie = lib.movie(user, movie_id)
    name = f"{get_normalized_title(movie)}.{movie.release_date[:4]}{os.path.splitext(video.path)[1]}"
    path = get_best_path(library_config["path"]["movie"], video.size)
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
    path = get_best_path(library_config["path"]["tv"], video.size)
    if path is None:
        raise Exception("no space left")

    for v in lib.videos(user).path(os.path.join(path, name)).results():
        v.full().delete()

    ensure_dir(os.path.join(path, f"{get_normalized_title(tv)}/"))
    shutil.copy(video.path, os.path.join(path, name))
    video.set_path(os.path.join(path, name))



class TvEssentialHandler:
    @staticmethod
    async def GET(server, user: "usr"):
        library = server.get_user_data("medialib")
        recent = library.tvs(user["name"]).order_by("MAX(Episodes.release_date) DESC").json_results(30)
        last = library.tvs(user["name"]).order_by("MAX(Videos.adding) DESC").json_results(30)
        txt = '{{"last":{last}, "recent":{recent}}}'
        return  txt.format(last = last, recent = recent).encode()


class MovieEssentialHandler:
    @staticmethod
    async def GET(server, user: "usr"):
        library = server.get_user_data("medialib")
        recent = library.movies(user["name"]).order_by("Movies.release_date DESC").json_results(30)
        last = library.movies(user["name"]).order_by("MAX(Videos.adding) DESC").json_results(30)
        txt = '{{"last":{last}, "recent":{recent}}}'
        return  txt.format(last = last, recent = recent).encode()


class TvGenreHandler:
    @staticmethod
    async def GET(server):
        return server.get_user_data("medialib").genre_tv_json().encode()


class MovieGenreHandler:
    @staticmethod
    async def GET(server):
        return server.get_user_data("medialib").genre_movie_json().encode()


class IndexHandler:
    @staticmethod
    async def GET():
        return {}


class MovieSearchHandler:
    @staticmethod
    async def GET(query: "str"):
        return Tmdb.search_movie_json(query).encode()

class TvSearchHandler:
    @staticmethod
    async def GET(query: "str"):
        return Tmdb.search_tv_json(query).encode()

class PersonHandler:
    @staticmethod
    async def GET(server, user: "usr", person_id: "url_1" = None):
        library = server.get_user_data("medialib")
        if person_id:
            person = library.person(user["name"], int(person_id))
            person.set_movie()
            person.set_tv()
            return person.json().encode()
        else:
            return library.persons(user["name"]).json_results().encode()

class MovieHandler:
    @staticmethod
    async def GET(server, user: "usr", movie_id: "url_1" = None):
        library = server.get_user_data("medialib")
        if movie_id:
            movie = library.movie(user["name"], int(movie_id))
            movie.set_videos()
            movie.set_persons()
            movie.set_trailers()
            return movie.json().encode()
        else:
            return library.movies(user["name"]).json_results().encode()

    @staticmethod
    async def PUT(server, movie_id: "url_1", input_data: "ipt", user: "usr"):
        library = server.get_user_data("medialib")
        if "watched" in input_data:
            library.movie(user["name"], int(movie_id)).set_watched(input_data["watched"])

class TvHandler:
    @staticmethod
    async def GET(server, user: "usr", tv_id: "url_1" = None, season_number: "url_2" = None, episode_number: "url_3" = None, episode_id: "int" = None):
        library = server.get_user_data("medialib")
        if episode_number:
            episode = library.tv_episode(user["name"],  int(tv_id), int(season_number),  int(episode_number))
            if episode is None:
                raise Exception("NotFound episode not found")
            episode.set_tv()
            episode.set_season()
            episode.set_videos()
            return episode.json().encode()

        elif season_number:
            season = library.tv_season(user["name"], int(tv_id), int(season_number))
            season.set_episodes()
            # season.set_episode_videos()
            season.set_tv()
            return season.json().encode()

        elif tv_id:
            tv = library.tv(user["name"], int(tv_id))
            tv.set_seasons()
            tv.set_persons()
            tv.set_trailers()
            return tv.json().encode()

        else:
            if episode_id is not None:
                return library.tv_episodes(user["name"]).id(episode_id).json_results().encode()
            return library.tvs(user["name"]).json_results().encode()

    @staticmethod
    async def PUT(server, tv_id: "url_1", season_id: "url_2", episode_id: "url_3", input_data: "ipt", user: "usr"):
        library = server.get_user_data("medialib")
        if len(tv_id) > 0:
            if "watched" in input_data:
                if len(tv_id) > 0 and len(season_id) > 0 and len(episode_id) > 0:
                    library.tv_episode(user["name"], int(tv_id), int(season_id), int(episode_id)).set_watched(input_data["watched"])
                elif len(tv_id) > 0 and len(season_id) > 0:
                    library.tv_season(user["name"], int(tv_id), int(season_id)).set_watched(input_data["watched"])
                elif len(tv_id) > 0:
                    library.tv(user["name"], int(tv_id)).set_watched(input_data["watched"])


class StreamHandler:
    @staticmethod
    async def GET(server, id: "url_1"):
        library = server.get_user_data("medialib")
        return {"path": library.video("reader", int(id)).path}


class CollectionHandler:
    @staticmethod
    async def GET(server, user: "usr", id: "url_1" = None, restrict: "int" = 0):

        library = server.get_user_data("medialib")

        if id:
            collection = library.collection(user["name"], int(id))
            collection.set_movie()
            collection.set_tv()
            return collection.json().encode()
        else:
            collections = library.collections(user)
            if restrict:
                collections.restrict()

            return collections.json_results().encode()

    @staticmethod
    async def POST(server, input_data: "ipt", user: "usr"):

        library = server.get_user_data("medialib")

        collection = library.new_collection(user["name"], input_data['name'])
        collection.edit_description(input_data['description'])
        collection.save()
        return {}

    @staticmethod
    async def PUT(server, id: "url_1", input_data: "ipt", user: "usr"):

        library = server.get_user_data("medialib")
        collection = library.collection(user["name"], int(id))

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
        return {}


class VideoHandler:
    @staticmethod
    async def GET(server, user: "usr", id: "url_1" = None, media_id: "int"=None, media_type: "str"=None):
        library = server.get_user_data("medialib")
        if id:
            video = library.video(user["name"], int(id))
            return video.json().encode()
        else:
            search = library.videos(user["name"])
            if media_id:
                search.media_id(media_id)
            if media_type == "movie":
                search.movie()
            if media_type == "tv":
                search.tv()
            return search.json_results().encode()

    @staticmethod
    async def POST(server, input_data: "ipt"):
        library = server.get_user_data("medialib")
        print(input_data)
        user = "maker"
        if "movieID" in input_data:
            await server.get_user_data("executor")(movie_maker, user, library, input_data['path'], input_data['movieID'])
        elif "tvID" in input_data:
            await server.get_user_data("executor")(tv_maker, user, library, input_data['path'], input_data['tvID'],
                          input_data['season'], input_data['episode'])
        else:
            raise Exception(f"media info invalid {input_data}")
        return {'Ok': True}

    @staticmethod
    async def PUT(server, id: "url_1", input_data: "ipt", user: "usr"):
        library = server.get_user_data("medialib")

        video = library.video(user["name"], int(id))

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

        return {}

    @staticmethod
    async def DELETE(server, id: "url_1", user: "usr"):
        library = server.get_user_data("medialib")

        if user["name"] != "root" and user["name"] != "jief":
            raise Exception("Not allowed")

        video = library.video(user["name"], int(id))
        if video is None:
            raise Exception("Error video not found")
        video.delete()
        os.remove(video.path)

        return {}