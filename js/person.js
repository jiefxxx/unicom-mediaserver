app.controller('personTable', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {
    $scope.pageSize = 50;
    $scope.searchText = "";
    $scope.searchGenre = "";
    $scope.genres = [];
    $scope.orderValue = "name";
    $scope.$watch('searchText', function() { $scope.pageSize = 50; $window.scrollTo(0, 0);}, true);
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

    $http.get("/MediaServer/api/person")
    .then(function (response) {
        videos = response.data;
        $scope.collection = videos;
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
        if ($scope.searchText.length > 0){
            if (!item.name.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, "").includes(
                    $scope.searchText.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, ""))){
                return false;
            }
        }

        return true;
    };


}]);