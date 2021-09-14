use std::collections::BTreeSet;

pub struct CharsetRequest {
    pub ansi: bool,
    pub schinese1: bool,
    pub schinese2: bool,
    pub schinese3: bool
}

impl CharsetRequest {
    pub fn new() -> Self {
        Self {
            ansi: true,
            schinese1: false,
            schinese2: false,
            schinese3: false
        }
    }

    pub fn get_charset(&self) -> BTreeSet<char> {
        let mut s = String::new();

        if self.ansi {
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