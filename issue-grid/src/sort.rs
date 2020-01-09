
use std::{
    collections::HashSet,
    fmt::Debug,
};

use regex::Regex;

/// Executable instruction for organizing issues.
pub enum OrganizeInstr {
    Bin(IssueSortInstr),
    Sort(IssueSortInstr),
}

/// Instruction for binning, and possibly sorting, instructions.
pub struct IssueSortInstr {
    pub filter: Regex,
    /// Explicit manual organization is possible.
    pub sorter: Option<Vec<Regex>>,
}

impl IssueSortInstr {
    /// Filters and organizes elements.
    pub fn sort<T, M, D, E>(
        &self,
        mut elems: Vec<T>,
        matches: M,
        human_identifier: E
    ) -> Vec<T>
    where
        M: Fn(&T, &Regex) -> bool,
        D: Debug,
        E: Fn(&T) -> &D,
    {
        // filter
        elems.retain(|elem| matches(elem, &self.filter));

        // sort
        if let Some(ref order) = self.sorter {
            let mut by_match: Vec<(T, Option<usize>)> = elems.into_iter()
                .map(|elem| {
                    let i: Option<usize> = order.iter()
                        .enumerate()
                        .find(|&(_, regex)| matches(&elem, regex))
                        .map(|(i, _)| i);

                    if i.is_none() {
                        eprintln!("[warn] elem didn't match any regex: {:?}",
                                  human_identifier(&elem));
                    }

                    (elem, i)
                })
                .collect();

            by_match.sort_by_key(|&(_, i)| i);
            by_match.into_iter()
                .map(|(elem, _)| elem)
                .collect()
        } else {
            elems
        }
    }
}


// TODO: this is incorrect

/// Bin and sort by an array of IssueSortInstr.
pub fn bin_sort<'a, T, M, V0, V1>(sorters: V0, elems: V1, tag_matcher: M) -> Vec<Vec<T>>
    where
        T: Clone + Debug + 'static,
        M: Fn(&T, &Regex) -> Vec<String>,
        V0: IntoIterator<Item = &'a IssueSortInstr>,
        V1: IntoIterator<Item = &'a T> + Clone,
{
    // bin
    let bins: Vec<Vec<(usize, T)>> = sorters.into_iter()
        .map(|sorter| {
            let bin: Vec<(usize, T, Vec<String>)> = elems.clone().into_iter()
                .cloned()
                .enumerate()
                .filter_map(|(index, elem)| {
                    let tags = tag_matcher(&elem, &sorter.filter);
                    if tags.len() > 0 {
                        Some((index, elem, tags))
                    } else {
                        None
                    }
                })
                .collect();

            let bin: Vec<(usize, T, Vec<String>)> =
                sorter.sort(
                    bin,
                    |&((_, _, &tags), ref regex)| {
                        tags.iter()
                            .any(|tag| regex.is_match(tag))
                    },
                    |&(_, _, &tags)| tags,
                );
            bin
        })
        .collect();

    // warn on duplicates
    let mut encountered_elems: HashSet<usize> = HashSet::new();
    for bin in &bins {
        for &(elem_i, ref elem, _) in bin {
            // hacky but whatever
            let mut duplicate = true;
            encountered_elems.get_or_insert_with(
                &elem_i,
                |_| {
                    duplicate = false;
                    elem_i
                }
            );
            if duplicate {
                eprintln!("[warn] elem sorted into several bins: {:?}", elem);
            }
        }
    }

    // return
    bins.into_iter()
        .map(|bin| bin
            .into_iter()
            .map(|(_, elem, _)| elem)
            .collect()
        )
        .collect()
}