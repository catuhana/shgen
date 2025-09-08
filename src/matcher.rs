use aho_corasick::{AhoCorasick, AhoCorasickBuilder};

use crate::{
    config::{SearchConfig, SearchFields},
    openssh_format::{Fingerprint, OpenSSHFormatter, OpenSSHPrivateKey, OpenSSHPublicKey},
};

pub struct Matcher {
    search: SearchConfig,

    aho_corasick: AhoCorasick,
}

impl Matcher {
    pub fn new(keywords: Vec<String>, search: SearchConfig) -> Self {
        let aho_corasick = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(keywords)
            .unwrap();

        Self {
            search,
            aho_corasick,
        }
    }

    pub fn search_matches(
        &self,
        openssh_formatter: &mut OpenSSHFormatter,
    ) -> Option<(OpenSSHPublicKey, OpenSSHPrivateKey)> {
        let fields = &self.search.fields;

        let match_found = if self.search.matching.all_fields {
            fields
                .iter()
                .all(|field| self.search_in_field(field, openssh_formatter))
        } else {
            fields
                .iter()
                .any(|field| self.search_in_field(field, openssh_formatter))
        };

        if match_found {
            // When a match is found, we'll stop anyway
            // so it's fine to re-format the keys here.
            let public_key = openssh_formatter.format_public_key();
            let private_key = openssh_formatter.format_private_key();

            Some((public_key, private_key))
        } else {
            None
        }
    }

    fn search_in_field(
        &self,
        field: &SearchFields,
        openssh_formatter: &mut OpenSSHFormatter,
    ) -> bool {
        match field {
            SearchFields::PublicKey => {
                let public_key = openssh_formatter.format_public_key();
                self.matches_aho_corasick(&public_key)
            }
            SearchFields::PrivateKey => {
                let private_key = openssh_formatter.format_private_key();
                self.matches_aho_corasick(&private_key)
            }
            fingerprint => {
                let fingerprint_type = match fingerprint {
                    SearchFields::Sha1Fingerprint => Fingerprint::Sha1,
                    SearchFields::Sha256Fingerprint => Fingerprint::Sha256,
                    SearchFields::Sha384Fingerprint => Fingerprint::Sha384,
                    SearchFields::Sha512Fingerprint => Fingerprint::Sha512,
                    _ => unreachable!(),
                };

                let fingerprint = openssh_formatter.format_fingerprint(&fingerprint_type);
                self.matches_aho_corasick(&fingerprint)
            }
        }
    }

    #[inline]
    fn matches_aho_corasick(&self, haystack: &str) -> bool {
        if !self.search.matching.all_keywords {
            return self.aho_corasick.is_match(haystack);
        }

        let patterns = self.aho_corasick.patterns_len();

        let mut seen_bits = 0u64;
        let target_bits = (1u64 << patterns) - 1;

        for mat in self.aho_corasick.find_iter(haystack) {
            let id = mat.pattern().as_usize();
            let bit = 1u64 << id;

            seen_bits |= bit;

            if seen_bits == target_bits {
                return true;
            }
        }

        false
    }
}
