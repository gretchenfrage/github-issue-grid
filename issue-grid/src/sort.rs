
use std::{
    convert::identity,
    iter::FromIterator,
};

use regex::Regex;

pub trait RegexMatch {
    fn is_match(&self, regex: &Regex) -> bool;
}

#[derive(Debug, Clone)]
pub struct PatternList<M> {
    pub patterns: Vec<(Regex, M)>,
}

impl<M> PatternList<M> {
    pub fn bin<E, I>(&self, elems: I, duplicate: bool) -> Binned<E, &M>
    where
        E: RegexMatch + Clone,
        I: IntoIterator<Item=E>,
    {
        let mut options: Vec<Option<E>> = elems.into_iter().map(Some).collect();
        let bins = self.patterns.iter()
            .map(|&(ref regex, ref meta)| {
                let bin = options.iter_mut()
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
                    .collect::<Vec<E>>();
                (bin, meta)
            })
            .collect();
        let overflow = options.into_iter()
            .flat_map(identity)
            .collect();

        Binned {
            bins,
            overflow,
        }
    }

    pub fn sort<E, I>(&self, elems: I) -> Vec<E>
    where
        E: RegexMatch + Clone,
        I: IntoIterator<Item=E>,
    {
        let Binned {
            bins,
            overflow
        } = self.bin(elems, false);

        let len = bins.iter()
            .map(|&(ref bin, _)| bin.len())
            .sum::<usize>()
            + overflow.len();
        let mut vec = Vec::with_capacity(len);
        vec.extend(bins.into_iter().flat_map(|(bin, _)| bin));
        vec.extend(overflow);

        vec
    }
}

pub struct Binned<E, M> {
    pub bins: Vec<(Vec<E>, M)>,
    pub overflow: Vec<E>,
}

impl<'a, E, M: Clone> Binned<E, &'a M> {
    pub fn cloned(self) -> Binned<E, M> {
        let Binned { bins, overflow } = self;
        let bins = bins.into_iter()
            .map(|(vec, meta)| (vec, meta.clone()))
            .collect();
        Binned { bins, overflow }
    }
}

impl<M> FromIterator<(Regex, M)> for PatternList<M> {
    fn from_iter<T: IntoIterator<Item=(Regex, M)>>(iter: T) -> Self {
        PatternList {
            patterns: iter.into_iter().collect(),
        }
    }
}

