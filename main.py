import asyncio
import json
import os
import toml

config = toml.load("./config.toml")

from unicom import Server, UnicomException
import medialibrary
from medialibrary import Library, Tmdb

medialibrary.tmdb_init(config["tmdb"]["key"], config["tmdb"]["language"])


async def shell(cmd):
    proc = await asyncio.create_subprocess_shell(cmd)
    await proc.communicate()
    return proc.returncode


class TvEssentialHandler:
    @staticmethod
    async def GET(server, user: str):
        last = server.library.tvs(user).order_by("MAX(Episodes.release_date)").json_results(30)
        recent = server.library.tvs(user).order_by("MAX(Videos.adding)").json_results(30)
        return f'{{ "last":{last}, "recent":{recent}}}'.encode()


class MovieEssentialHandler:
    @staticmethod
    async def GET(server, user:str):
        last = server.library.movies(user).order_by("MAX(Movies.release_date)").json_results(30)
        recent = server.library.movies(user).order_by("MAX(Videos.adding)").json_results(30)
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
    async def POST(input_data: "Json"):
        if "movieID" in input_data:
            ret = await shell(f"python3 local_maker.py '{input_data['path']}' movie {input_data['movieID']}")
        elif "tvID" in input_data:
            ret = await shell(f"python3 local_maker.py '{input_data['path']}' tv {input_data['tvID']} {input_data['season']} {input_data['episode']}")
        else:
            raise Exception(f"media info invalid {input_data}")
        if ret < 0:
            raise Exception(f"local_maker error code :{ret}")
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
        if user != "root":
            raise Exception("Not allowed")
        if len(url[1]) > 0:
            video_id = int(url[1])
            video = server.library.get_video(video_id)
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
