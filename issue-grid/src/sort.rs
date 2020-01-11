
use std::convert::identity;

use regex::Regex;

pub trait RegexMatch {
    fn is_match(&self, regex: &Regex) -> bool;
}

pub struct PatternList {
    pub patterns: Vec<Regex>,
}

impl PatternList {
    pub fn bin<T, I>(&self, elems: I, duplicate: bool) -> Binned<T>
    where
        T: RegexMatch + Clone,
        I: IntoIterator<Item=T>,
    {
        let mut options: Vec<Option<T>> = elems.into_iter().map(Some).collect();
        let bins = self.patterns.iter()
            .map(|regex| options.iter_mut()
                .flat_map(|option| {
                    let is_match = match option.as_ref() {
                        Some(elem) => elem.is_match(regex),
                        None => false,
                    };
                    match is_match {
                        true if duplicate => Some(option.take().unwrap()),
                        true => Some(option.clone().unwrap()),
                        false => None,
                    }
                })
                .collect::<Vec<T>>())
            .collect();
        let overflow = options.into_iter()
            .flat_map(identity)
            .collect();

        Binned {
            bins,
            overflow,
        }
    }

    pub fn sort<T, I>(&self, elems: I) -> Vec<T>
    where
        T: RegexMatch + Clone,
        I: IntoIterator<Item=T>,
    {
        let Binned {
            bins,
            overflow
        } = self.bin(elems, false);

        let len = bins.iter().map(Vec::len).sum::<usize>() + overflow.len();
        let mut vec = Vec::with_capacity(len);
        vec.extend(bins.into_iter().flat_map(identity));
        vec.extend(overflow);

        vec
    }
}

pub struct Binned<T> {
    pub bins: Vec<Vec<T>>,
    pub overflow: Vec<T>,
}
