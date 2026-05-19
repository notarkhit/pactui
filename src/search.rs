use crate::package::Package;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    NameFuzzy,
    NameExact,
    DescFuzzy,
    DescExact,
}

impl SearchMode {
    pub fn next(self) -> Self {
        match self {
            SearchMode::NameFuzzy => SearchMode::NameExact,
            SearchMode::NameExact => SearchMode::DescFuzzy,
            SearchMode::DescFuzzy => SearchMode::DescExact,
            SearchMode::DescExact => SearchMode::NameFuzzy,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SearchMode::NameFuzzy => "name-fuzzy",
            SearchMode::NameExact => "name-exact",
            SearchMode::DescFuzzy => "desc-fuzzy",
            SearchMode::DescExact => "desc-exact",
        }
    }
}

pub fn filter(packages: &[Package], query: &str, mode: SearchMode) -> Vec<Package> {
    if query.is_empty() {
        return packages.to_vec();
    }

    let use_parallel = packages.len() > 10_000;

    match mode {
        SearchMode::NameFuzzy => {
            let matcher = SkimMatcherV2::default();
            if use_parallel {
                let mut scored: Vec<(i64, Package)> = packages
                    .par_iter()
                    .filter_map(|p| {
                        matcher
                            .fuzzy_match(&p.name, query)
                            .map(|score| (score, p.clone()))
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.cmp(&a.0));
                scored.into_iter().map(|(_, p)| p).collect()
            } else {
                let mut scored: Vec<(i64, Package)> = packages
                    .iter()
                    .filter_map(|p| {
                        matcher
                            .fuzzy_match(&p.name, query)
                            .map(|score| (score, p.clone()))
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.cmp(&a.0));
                scored.into_iter().map(|(_, p)| p).collect()
            }
        }
        SearchMode::NameExact => {
            let q = query.to_lowercase();
            packages
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
        SearchMode::DescFuzzy => {
            let matcher = SkimMatcherV2::default();
            if use_parallel {
                let mut scored: Vec<(i64, Package)> = packages
                    .par_iter()
                    .filter_map(|p| {
                        let haystack = p
                            .description
                            .as_deref()
                            .unwrap_or("")
                            .to_string()
                            + " "
                            + &p.name;
                        matcher
                            .fuzzy_match(&haystack, query)
                            .map(|score| (score, p.clone()))
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.cmp(&a.0));
                scored.into_iter().map(|(_, p)| p).collect()
            } else {
                let mut scored: Vec<(i64, Package)> = packages
                    .iter()
                    .filter_map(|p| {
                        let haystack = p
                            .description
                            .as_deref()
                            .unwrap_or("")
                            .to_string()
                            + " "
                            + &p.name;
                        matcher
                            .fuzzy_match(&haystack, query)
                            .map(|score| (score, p.clone()))
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.cmp(&a.0));
                scored.into_iter().map(|(_, p)| p).collect()
            }
        }
        SearchMode::DescExact => {
            let q = query.to_lowercase();
            packages
                .iter()
                .filter(|p| {
                    p.description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                        || p.name.to_lowercase().contains(&q)
                })
                .cloned()
                .collect()
        }
    }
}
