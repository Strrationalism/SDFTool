use std::{collections::BTreeSet};

pub struct CharsetRequest {
    pub ascii: bool,
    pub schinese1: bool,
    pub schinese2: bool,
    pub schinese3: bool
}

impl CharsetRequest {
    pub fn from_args(arg: &clap::ArgMatches) -> Self {
        let mut x = Self::new();

        if arg.is_present("no-ascii") {
            x.ascii = false;
        }

        if arg.is_present("schinese1") {
            x.schinese1 = true;
        }

        if arg.is_present("schinese2") {
            x.schinese2 = true;
        }

        if arg.is_present("schinese3") {
            x.schinese3 = true;
        }

        x
    }

    pub fn new() -> Self {
        Self {
            ascii: true,
            schinese1: false,
            schinese2: false,
            schinese3: false
        }
    }

    pub fn get_charset(&self) -> BTreeSet<char> {
        let mut s = String::new();

        if self.ascii {
            s.push_str(include_str!("./charset/ascii.txt"));
        }

        if self.schinese1 {
            s.push_str(include_str!("./charset/common-standard-chinese-characters-table/level-1.txt"));
        }

        if self.schinese2 {
            s.push_str(include_str!("./charset/common-standard-chinese-characters-table/level-2.txt"));
        }

        if self.schinese3 {
            s.push_str(include_str!("./charset/common-standard-chinese-characters-table/level-3.txt"));
        }

        s.chars().filter(|x| *x != '\n' && *x != '\r').collect()
    }
}

#[test]
fn test_charset() {
    let mut req = CharsetRequest::new();
    req.schinese1 = true;
    req.schinese2 = true;
    req.schinese3 = true;

    for i in req.get_charset() {
        print!("{}", i);
    }
}
