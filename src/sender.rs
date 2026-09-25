pub struct Senders(Vec<String>);

impl Senders {
    pub fn parse(list: &str) -> Senders {
        Senders(
            list.split(',')
                .map(str::trim)
                .filter(|pattern| !pattern.is_empty())
                .map(str::to_lowercase)
                .collect(),
        )
    }

    pub fn allows(&self, address: &str) -> bool {
        let address = address.to_lowercase();
        self.0.iter().any(|pattern| matches(pattern, &address))
    }
}

fn matches(pattern: &str, address: &str) -> bool {
    let mut parts: Vec<&str> = pattern.split('*').collect();
    let first = parts.remove(0);
    let Some(mut rest) = address.strip_prefix(first) else {
        return false;
    };
    let Some(last) = parts.pop() else {
        return rest.is_empty();
    };
    for part in parts {
        let Some(start) = rest.find(part) else {
            return false;
        };
        rest = &rest[start + part.len()..];
    }
    rest.ends_with(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_addresses_ignoring_case_and_spaces() {
        let senders = Senders::parse(" Me@Proton.me , other@example.com");

        assert!(senders.allows("me@proton.me"));
        assert!(senders.allows("OTHER@example.com"));
        assert!(!senders.allows("me@proton.me.evil.com"));
        assert!(!senders.allows("xme@proton.me"));
    }

    #[test]
    fn matches_wildcards_anywhere_in_the_pattern() {
        let senders = Senders::parse("*@proton.me,bills-*@*.example.com");

        assert!(senders.allows("anyone@proton.me"));
        assert!(senders.allows("bills-power@mail.example.com"));
        assert!(!senders.allows("anyone@proton.me.evil.com"));
        assert!(!senders.allows("bills-power@example.com"));
        assert!(!senders.allows("news@mail.example.com"));
    }

    #[test]
    fn allows_nobody_when_the_list_is_empty() {
        assert!(!Senders::parse(" , ").allows("me@proton.me"));
    }

    #[test]
    fn allows_everybody_with_a_lone_wildcard() {
        assert!(Senders::parse("*").allows("anyone@anywhere.com"));
    }
}
