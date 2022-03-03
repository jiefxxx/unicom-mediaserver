import asyncio
import os
import toml

config = toml.load("./config.toml")

from unicom import Server, UnicomException
import medialibrary
from medialibrary import Library, Tmdb

medialibrary.tmdb_init(config["tmdb"]["key"], config["tmdb"]["language"])

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
        print(user)
        if len(url[1]) > 0:
            person = server.library.person(user, int(url[1]))
            person.set_movie()
            person.set_tv()
            print(person)
            return person.json().encode()
        else:
            return server.library.persons(user).json_results().encode()

class MovieHandler:
    @staticmethod
    async def GET(server, url, user: str):
        print(user)
        if len(url[1]) > 0:
            movie = server.library.movie(user, int(url[1]))
            movie.set_videos()
            movie.set_persons()
            movie.set_trailers()
            print(movie)
            return movie.json().encode()
        else:
            return server.library.movies(user).json_results().encode()

    @staticmethod
    async def PUT(server, url, input_data: "Json", user: str):
        if len(url[1]) > 0:
            if "watched" in input_data:
                print(f'edit {input_data["watched"]}')
                server.library.movie(user, int(url[1])).set_watched(input_data["watched"])

class TvHandler:
    @staticmethod
    async def GET(server, url, user: str):
        print(url)
        if len(url) >= 4 and len(url[1]) > 0 and len(url[2]) > 0 and len(url[3]) > 0:
            episode = server.library.tv_episode(user,  int(url[1]), int(url[2]),  int(url[3]))
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
    async def GET():
        return b"{}"


class CollectionHandler:
    @staticmethod
    async def GET(server, url, user: str, restrict: int = 0):
        print(user)
        if len(url[1]) > 0:
            collection = server.library.collection(user, int(url[1]))
            collection.set_movie()
            collection.set_tv()
            return collection.json().encode()
        else:
            collections = server.library.collections(user)
            print(restrict)
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
    async def GET(server, url, user: str):
        if len(url[1]) > 0:
            video = server.library.video(user, int(url[1]))
            return video.json().encode()
        else:
            return server.library.videos(user).json_results().encode()

    @staticmethod
    async def PUT(server, url, input_data: "Json", user: str):
        print(url)

        if user != "root":
            raise Exception("Not allowed")

        if len(url[1]) == 0:
            raise Exception("Error no video id")

        video = server.library.video(user, int(url[1]))

        if video is None:
            raise Exception("Error video not found")

        if url[2] == "edit_media":
            if video.media_type == 0:
                video.set_movie(input_data["movie_id"])
            elif video.media_type == 1:
                video.set_tv(input_data["tv_id"],
                             input_data["season_number"],
                             input_data["episode_number"])

        elif url[2] == "set_watch_time":
            video.set_watch_time(input_data["current_time"])

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
    s.add_api(r"^/api/tv(?:/(\d+))?(?:/season/(\d+))?(?:/episode/(\d+))?$", TvHandler)
    s.add_api(r"^/api/person(?:/(\d+))?$", PersonHandler)
    s.add_api(r"^/api/collection(?:/(\d+))?$", CollectionHandler)
    s.add_api(r"^/api/moviesearch$", MovieSearchHandler)
    s.add_api(r"^/api/tvsearch$", TvSearchHandler)

    s.add_dynamic_file(r"^/stream(?:/(\d+))$", StreamHandler)

    s.add_static_file(r"^/js/(.*\.js)$", "js/")
    s.add_static_file(r"^/rsc/(.*)$", config["rsc"]+"/")
    asyncio.run(s.serve_forever())
