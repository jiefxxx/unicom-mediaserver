
app.controller('fileTable', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {
    $scope.searchText = "";
    $scope.itemsByPage=100;
    $scope.$watch('searchText', function() { $window.scrollTo(0, 0);}, true);

    $http.get("/mediaserver/api/video")
    .then(function (response) {
        videos = response.data;
        $scope.rowCollection = videos;
        console.log(videos);
    });

    $scope.scroll = function(){
        $window.scrollTo(0, 0);
        console.log("scroll");
    }

    $scope.convertPath = function(value){
        return value.split("/").pop()
    };
    $scope.convertInfo = function(value){
        if (value["Tv"]){
            return value["Tv"]["title"]+"S"+String(value["Tv"]["season_number"]).padStart(2, '0')+"E"+String(value["Tv"]["episode_number"]).padStart(2, '0')
        }
        else if (value["Movie"]){
            return value["Movie"]["title"]+" "+value["Movie"]["release_date"].split("-")[0]
        }
        return ""
    };
    $scope.convertInfoPath = function(value){
        if (value["Tv"]){
            return "tv/"+value["Tv"]["id"]+"/season/"+String(value["Tv"]["season_number"])+"/episode/"+String(value["Tv"]["episode_number"])
        }
        else if (value["Movie"]){
            return "movie/"+value["Movie"]["id"]
        }
    };
    $scope.deleteVideos = function(){
        selection = getSelection($scope.rowCollection);
        if (confirm("Voulez vous effacer")) {
             for (i = 0; i < selection.length; i++) {
                 var id = selection[i].id;
                 $http({
                    method: 'delete',
                    url: '/mediaserver/api/video/'+ id }).then(function (response) {
                        console.log("yeah", response.data)
                    });
             }
        }
    }

    $scope.editVideos = function(){
        var type = -1;
        selection = getSelection($scope.rowCollection);

        for (i = 0; i < selection.length; i++) {
            if (selection[i].media_type != type){
                if (type == -1){
                    type = selection[i].media_type
                }
                else{
                    alert("selection incompatible");
                    return
                }
            }
        }
        if (type == 0){
            for(var i=0;i<selection.length;i++){
                var id = selection[i].id;
                getModalMovie($uibModal, selection[i]).then(function (movie_id) {
                    var dataToPost = {"movie_id": parseInt(movie_id)};
                    $http({
                        method: 'PUT',
                        url: '/mediaserver/api/video/'+ id+ '/edit_media',
                        data: dataToPost }).then(function (response) {
                            console.log("yeah", response.data)
                        });
                }, function(value){
                });
            }
        }
        else if (type == 1){
            getModalTvShow($uibModal, selection).then(function (files){
                for(var i=0;i<selection.length;i++){
                    if (files[i].Season > -1 && files[i].Episode > -1 && files[i].MediaId > 0){
                        var dataToPost = {  "tv_id": parseInt(files[i].MediaId),
                                            "season_number": parseInt(files[i].Season),
                                            "episode_number": parseInt(files[i].Episode)};
                        $http({
                            method: 'PUT',
                            url: '/mediaserver/api/video/'+selection[i].id+'/edit_media',
                            data: dataToPost }).then(function (response) {
                                console.log("yeah",response.data)
                            });
                    }
                }

            }, function(value){
            });
        }
    }

}]);