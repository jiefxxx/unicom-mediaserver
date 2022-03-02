getModalCreateCollection = function($uibModal){
    var modalInstance = $uibModal.open({
        animation: true,
        ariaLabelledBy: 'modal-title',
        ariaDescribedBy: 'modal-body',
        templateUrl: '/mediaserver/modal/create_collection',
        controller: 'ModalCreateCollectionCtrl',
        controllerAs: 'pc',
        windowClass: 'show',
        backdropClass: 'show',
        size: 'lg'
    });
    return modalInstance.result;
}

app.controller('ModalCreateCollectionCtrl', function ($scope, $uibModalInstance, $http) {
    var pc = this;

    $scope.title = ""
    $scope.description = ""

    pc.ok = function () {
        var dataToPost = {"name": $scope.title,
                          "description": $scope.description};
        $http({
            method: 'POST',
            url: '/mediaserver/api/collection',
            data: dataToPost }).then(function (response) {
                $uibModalInstance.close("yeah");
            }).then(function (error){
                $uibModalInstance.dismiss("oooohh");
            });
    };

    pc.cancel = function () {
      $uibModalInstance.dismiss('cancel');
    };

  });


app.controller('collectionTable', ['$scope', '$http', '$uibModal', '$window',function ($scope, $http, $uibModal, $window) {
    $scope.pageSize = 50;
    $scope.searchText = "";
    $scope.searchCreator = "";
    $scope.creators = [];
    $scope.orderValue = "name";
    $scope.$watch('searchText', function() { $scope.pageSize = 50; $window.scrollTo(0, 0);}, true);
    $scope.$watch('searchCreator', function() { $scope.pageSize = 50; $window.scrollTo(0, 0);}, true);
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

    $http.get("/mediaserver/api/collection")
    .then(function (response) {
        videos = response.data;
        $scope.collection = videos;
        for (video in videos){
            if (!$scope.creators.includes(videos[video].creator)){
                    $scope.creators.push(videos[video].creator);
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
    $scope.createCollection = function (){
        getModalCreateCollection($uibModal).then(function (files){

        })
    }

    $scope.movieFilter = function (item) {

        if ($scope.searchCreator.length > 0){
            if (!item.creator == $scope.searchCreator){
                return false;
            }
        }

        if ($scope.searchText.length > 0){
            if (!item.name.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, "").includes(
                    $scope.searchText.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, ""))){
                return false;
            }
        }

        return true;
    };


}]);