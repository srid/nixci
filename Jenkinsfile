// We use https://github.com/juspay/jenkins-nix-ci

pipeline {
    agent any
    stages {
        stage ('Build') {
            steps {
                sh '''
                    # Build nixci, and then use it to build this project

                    # Sandbox must be disabled for integration test (uses nix)
                    # TODO: Need to make `nix run` work on Linux, still.

                    nix --option sandbox false build 
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
