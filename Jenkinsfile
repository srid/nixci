// We use https://github.com/juspay/jenkins-nix-ci

pipeline {
    agent any
    stages {
        stage ('Build') {
            steps {
                nixCI ()
            }
        }
        stage ('Cachix push') {
            when { branch 'nixci' }
            steps {
                cachixPush "srid"
            }
        }
    }
}
