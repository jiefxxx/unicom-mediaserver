app.controller('tvController', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {

    $scope.add_to_collection = function (tv_id){
        getModalGetCollection($uibModal).then(function(collection_id){
            var dataToPost = {"tv_id": parseInt(tv_id)};
            $http({
                method: 'PUT',
                url: '/MediaServer/api/collection/'+ collection_id,
                data: dataToPost }).then(function (response) {
                    console.log("yeah", response.data)
                });
        })
    }

}]);

app.controller('tvTable', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {
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

    $http.get("/MediaServer/api/tv")
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

    $scope.tvFilter = function (item) {

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