#[cfg(feature = "integration_test")]
mod integration_test {
    use clap::Parser;
    use nixci::{self, cli, nix::devour_flake::DrvOut};
    use regex::Regex;

    #[test]
    fn test_haskell_multi_nix() -> anyhow::Result<()> {
        let args = cli::CliArgs::parse_from(&[
            "nixci",
            "-v",
            "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321",
        ]);
        let outs = nixci::nixci(args)?;
        let expected = vec![
            "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0/bin/bar",
            "/nix/store/3x2kpymc1qmd05da20wnmdyam38jkl7s-ghc-shell-for-packages-0",
            // Yes, we have a duplicate. See https://github.com/srid/nixci/issues/18
            "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0",
            "/nix/store/hsj8mwn9vzlyaxzmwyf111scisnjhlkb-bar-0.1.0.0",
            "/nix/store/dzhf0i3wi69568m5nvyckck8bbs9yrfd-foo-0.1.0.0",
        ]
        .into_iter()
        .map(|s| DrvOut(s.to_string()))
        .collect::<Vec<_>>();
        assert_same_drvs(outs, expected);
        Ok(())
    }

    pub fn assert_same_drvs(drvs1: Vec<DrvOut>, drvs2: Vec<DrvOut>) {
        assert_eq!(drvs1.len(), drvs2.len());
        let mut drv1 = drvs1
            .into_iter()
            .map(|d| without_hash(&d))
            .collect::<Vec<_>>();
        let mut drv2 = drvs2
            .into_iter()
            .map(|d| without_hash(&d))
            .collect::<Vec<_>>();
        drv1.sort();
        drv2.sort();
        assert_eq!(drv1, drv2);
    }

    pub fn without_hash(out: &DrvOut) -> String {
        let re = Regex::new(r".+\-(.+)").unwrap();
        let captures = re.captures(out.0.as_str()).unwrap();
        captures.get(1).unwrap().as_str().to_string()
    }
}
