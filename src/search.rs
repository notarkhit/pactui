use crate::package::Package;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    NameFuzzy,
    NameExact,
}

impl SearchMode {
    pub fn next(self) -> Self {
        match self {
            SearchMode::NameFuzzy => SearchMode::NameExact,
            SearchMode::NameExact => SearchMode::NameFuzzy,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SearchMode::NameFuzzy => "fuzzy",
            SearchMode::NameExact => "exact",
        }
    }
}

pub fn filter(packages: &[Package], query: &str, mode: SearchMode) -> Vec<Package> {
    if query.is_empty() {
        return packages.to_vec();
    }

    match mode {
        SearchMode::NameFuzzy => {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(i64, Package)> = if packages.len() > 10_000 {
                packages
                    .par_iter()
                    .filter_map(|p| {
                        matcher
                            .fuzzy_match(&p.name, query)
                            .map(|score| (score, p.clone()))
                    })
                    .collect()
            } else {
                packages
                    .iter()
                    .filter_map(|p| {
                        matcher
                            .fuzzy_match(&p.name, query)
                            .map(|score| (score, p.clone()))
                    })
                    .collect()
            };
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            scored.into_iter().map(|(_, p)| p).collect()
        }
        SearchMode::NameExact => {
            let q = query.to_lowercase();
            packages
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    }
}
