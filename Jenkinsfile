// We use https://github.com/juspay/jenkins-nix-ci

pipeline {
    agent any
    stages {
        stage ('Build') {
            steps {
                sh '''
                    # Build nixci, and then use it to build this project

                    nix build 
                    ./result/bin/nixci .
                '''
            }
        }
        stage ('Test') {
            steps {
                sh '''
                    nix develop -c sh -xc "cargo test --test integration_test --features integration_test -- --nocapture"
                '''
            }
        }
        stage ('Build (legacy)') {
            steps {
                nixCI () // TODO: Remove this after migrating nixci in jenkins-nix-ci (for cachixPush to work)
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
