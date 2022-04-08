getModalMovie = function($uibModal, video){
    var modalInstance = $uibModal.open({
        animation: true,
        ariaLabelledBy: 'modal-title',
        ariaDescribedBy: 'modal-body',
        templateUrl: '/mediaserver/modal/moviesearch',
        controller: 'ModalMovieCtrl',
        controllerAs: 'pc',
        windowClass: 'show',
        backdropClass: 'show',
        size: 'lg',
        resolve: {
            video: function () {
                return video;
            }
        }
    });
    return modalInstance.result;
}

getModalTvShow = function($uibModal, videos){
    var modalInstance = $uibModal.open({
        animation: true,
        ariaLabelledBy: 'modal-title',
        ariaDescribedBy: 'modal-body',
        templateUrl: '/mediaserver/modal/tvsearch',
        controller: 'ModalTvCtrl',
        controllerAs: 'pc',
        windowClass: 'show',
        backdropClass: 'show',
        size: 'lg',
        resolve: {
            videos: function () {
                return videos;
            }
        }
    });
    return modalInstance.result;
}

app.controller('ModalMovieCtrl', function ($scope, $uibModalInstance, $http, video) {
    var pc = this;
    pc.video = video;
    $scope.textSearch = ""
    pc.movies = []
    pc.selectMovieId = 0

    $scope.searchMovie = function(){
        if ($scope.textSearch.length>0){
            $http.get("/mediaserver/api/moviesearch?query="+$scope.textSearch)
            .then(function (response) {
                pc.movies = response.data.results;
                console.log("yeah",pc.movies)
            });
        }
    }

    pc.ok = function () {
      $uibModalInstance.close(pc.selectMovieId);

    };

    pc.cancel = function () {
      $uibModalInstance.dismiss('cancel');
    };

  });

  app.controller('ModalTvCtrl', function ($scope, $uibModalInstance, $http, videos) {
    var pc = this;
    var i;
    pc.videos = []
    for (i = 0; i < videos.length; i++) {
        var path = videos[i].path;
        if  (path == undefined){
            path = videos[i].Path
        }
        var result = path.match(/(.*)[sS](\d*)[eExX](\d*).*/);
        if (result){
            videos[i]["Season"] = parseInt(result[2]);
            videos[i]["Episode"] = parseInt(result[3]);
        }else{
            videos[i]["Season"] = -1;
            videos[i]["Episode"] = -1;
        }
        pc.videos.push(videos[i]);
    }

    $scope.textSearch = ""
    pc.tvShows = [];
    pc.selectTvShowsId = 0;

    find_SeasonEpisode = function(path){
        var result = path.Math(/(.*)[s](\d*)[e](\d*).*/);
        return (result[2], result[3])
    }

    $scope.searchTvShow = function(){
        if ($scope.textSearch.length>0){
            console.log($scope.textSearch)
            $http.get("/mediaserver/api/tvsearch?query="+$scope.textSearch)
            .then(function (response) {
                pc.tvShows = response.data.results;
                console.log(pc.tvShows)
            });
        }
    }

    pc.ok = function () {
        for(var i=0;i<pc.videos.length;i++){
            pc.videos[i].MediaId = pc.selectTvShowsId
        }
        $uibModalInstance.close(pc.videos);
    };

    pc.cancel = function () {
      $uibModalInstance.dismiss('cancel');
    };
  });

  getModalGetCollection = function($uibModal){
    var modalInstance = $uibModal.open({
        animation: true,
        ariaLabelledBy: 'modal-title',
        ariaDescribedBy: 'modal-body',
        templateUrl: '/mediaserver/modal/get_collection',
        controller: 'ModalGetCollectionCtrl',
        controllerAs: 'pc',
        windowClass: 'show',
        backdropClass: 'show',
        size: 'lg'
    });
    return modalInstance.result;
}

app.controller('ModalGetCollectionCtrl', function ($scope, $uibModalInstance, $http) {
    var pc = this;

    $scope.searchText = ""
    $scope.idSelected = null;
    $scope.nameSelected = null;

    $scope.local = JSON.parse(localStorage.getItem('collections'));

    $http.get("/mediaserver/api/collection?restrict=1")
    .then(function (response) {
        videos = response.data;
        $scope.collections = videos;
    });

    $scope.setSelected = function (idSelected, nameSelected) {
        $scope.idSelected = idSelected;
        $scope.nameSelected = nameSelected;
    };

    $scope.myFilter = function (item) {
        if ($scope.searchText.length > 0){
            if (!item.name.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, "").includes(
                    $scope.searchText.toLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, ""))){
                return false;
            }
        }

        return true;
    };

    pc.ok = function () {
        current = {"id": $scope.idSelected, "name": $scope.nameSelected};

        if ($scope.idSelected == null || $scope.nameSelected == null){
            $uibModalInstance.dismiss('nothing selected');
            return
        }

        if ($scope.local == null){
            $scope.local = [];
        }

        for( var i = 0; i < $scope.local.length; i++){

            if ( $scope.local[i].id === $scope.idSelected) {
                $scope.local.splice(i, 1);
            }

        }

        $scope.local.push(current);

        localStorage.setItem('collections', JSON.stringify($scope.local.slice(-3)));

        $uibModalInstance.close($scope.idSelected);
    };

    pc.cancel = function () {
      $uibModalInstance.dismiss('cancel');
    };

  });
