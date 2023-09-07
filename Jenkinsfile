// We use https://github.com/juspay/jenkins-nix-ci

pipeline {
    agent any
    stages {
        stage ('Build') {
            steps {
                sh '''
                    # Build nixci, and then use it to build this project
                    nix build
                    ./result/bin/nixci . -- --option sandbox false
                '''
            }
        }
        stage ('Cachix push') {
            when { branch 'master' }
            steps {
                cachixPush "srid"
            }
        }
    }
}
