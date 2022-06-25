app.controller('movieController', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {
    $scope.$watch('watched', function() {
        if ($scope.watched){
            $scope.watched_image = "/rsc/see-full.png"
        }
        else{
            $scope.watched_image = "/rsc/see-empty.png"
        }
    }, true);

    $scope.add_to_collection = function (movie_id){
        getModalGetCollection($uibModal).then(function(collection_id){
            var dataToPost = {"movie_id": parseInt(movie_id)};
            $http({
                method: 'PUT',
                url: '/MediaServer/api/collection/'+ collection_id,
                data: dataToPost }).then(function (response) {
                    console.log("yeah", response.data)
                });
        })
    }

    $scope.edit_movie = function(selection){
        console.log(selection);
        getModalMovie($uibModal, "test").then(function (movie_id) {
            var dataToPost = {"movie_id": parseInt(movie_id)};
            for(var i=0;i<selection.length;i++){
                $http({
                method: 'PUT',
                url: '/MediaServer/api/video/'+ selection[i],
                data: dataToPost }).then(function (response) {
                    console.log("yeah", response.data)
                });
            }

        }, function(value){

        });
    };

    $scope.delete_video = function(id){
        if (confirm("Voulez vous effacer")) {
             $http({
                method: 'delete',
                url: '/MediaServer/api/video/'+ id }).then(function (response) {
                    console.log("yeah", response.data)
                });
        }
    }

    $scope.set_watched = function(id){
        console.log("scope " + $scope.watched)
        $http({
            method: 'PUT',
            url: '/MediaServer/api/movie/'+ id ,
            data: {"watched": $scope.watched}}).then(function (response) {
                console.log("yeah", response.data)
            });
    }


}]);

app.controller('movieTable', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {
    $scope.pageSize = 50;
    $scope.searchText = "";
    $scope.searchGenre = "";
    $scope.genres = [];
    $scope.orderValue = "title";
    $scope.$watch('searchText', function() { $scope.pageSize = 50; $window.scrollTo(0, 0);}, true);
    $scope.$watch('searchGenre', function() { $scope.pageSize = 50; $window.scrollTo(0, 0);}, true);
    $scope.$watch('orderValue', function() { $scope.pageSize = 50; $window.scrollTo(0, 0);}, true);

    window.onscroll = function() {
        // @var int totalPageHeight
        var totalPageHeight = document.body.scrollHeight;

        // @var int scrollPoint
        var scrollPoint = window.scrollY + window.innerHeight;
        // check if we hit the bottom of the page
        if(scrollPoint >= totalPageHeight - 20)
        {
            $scope.pageSize = $scope.pageSize + 50;
            $scope.$digest();
        }
    }

    $http.get("/MediaServer/api/movie")
    .then(function (response) {
        videos = response.data;
        $scope.collection = videos;
        for (video in videos){
            for (genre in videos[video].genres){
                if (!$scope.genres.includes(videos[video].genres[genre])){
                    $scope.genres.push(videos[video].genres[genre]);
                }

            }
        }
        console.log(videos);
    });

    $scope.reversSelection = function () {
        if($scope.orderValue.startsWith("-")){
            $scope.orderValue = $scope.orderValue.substring(1);
        }
        else{
            $scope.orderValue = "-" + $scope.orderValue;
        }
    };

    $scope.movieFilter = function (item) {

        if ($scope.searchGenre.length > 0){
            if (!item.genres.includes($scope.searchGenre)){
                return false;
            }
        }

        if ($scope.searchText.length > 0){
            if (!item.title.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, "").includes(
                    $scope.searchText.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, ""))){
                return false;
            }
        }

        return true;
    };


}]);