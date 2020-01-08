
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
    pub fn sort<T, K>(&self, mut elems: Vec<T>, key: K) -> Vec<T>
        where
            K: Fn(&T) -> &str
    {
        // filter
        elems.retain(|elem| {
            self.filter.is_match(key(elem))
        });

        // sort
        if let Some(ref order) = self.sorter {
            let mut by_match: Vec<(T, Option<usize>)> = elems.into_iter()
                .map(|elem| {
                    let i: Option<usize> = order.iter()
                        .enumerate()
                        .find(|&(_, regex)| {
                            regex.is_match(key(&elem))
                        })
                        .map(|(i, _)| i);

                    if i.is_none() {
                        eprintln!("[warn] elem didn't match any regex: {:?}", key(&elem));
                    }

                    (elem, i)
                })
                .collect();

            by_match.sort_by_key(|&(_, i)| i);
            by_match.into_iter()
                .map(|(elem, _)| elem)
                .collect()
        } else {
            // fallback to alphabetical
            elems.sort_by_key(|elem| key(elem).to_string());
            elems
        }
    }
}

/// Bin and sort by an array of IssueSortInstr.
pub fn bin_sort<'a, T, K, V0, V1>(sorters: V0, elems: V1, key: K) -> Vec<Vec<T>>
    where
        T: Clone + Debug + 'static,
        K: Fn(&T) -> &str,
        V0: IntoIterator<Item = &'a IssueSortInstr>,
        V1: IntoIterator<Item = &'a T> + Clone,
{
    // bin
    let bins: Vec<Vec<(usize, T)>> = sorters.into_iter()
        .map(|sorter| {
            let bin: Vec<(usize, T)> =
                elems.clone().into_iter().cloned().enumerate().collect();
            let bin: Vec<(usize, T)> =
                sorter.sort(bin, |&(_, ref elem)| key(elem));
            bin
        })
        .collect();

    // warn on duplicates
    let mut encountered_elems: HashSet<usize> = HashSet::new();
    for bin in &bins {
        for &(elem_i, ref elem) in bin {
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
            .map(|(_, elem)| elem)
            .collect()
        )
        .collect()
}