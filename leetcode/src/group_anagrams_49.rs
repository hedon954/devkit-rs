#[allow(unused)]
fn group_anagrams(strs: Vec<String>) -> Vec<Vec<String>> {
    let mut tmp: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for item in strs.iter() {
        let mut i_chars: Vec<char> = item.chars().collect();
        i_chars.sort_unstable();
        let s: String = i_chars.iter().collect();
        match tmp.get_mut(&s) {
            Some(list) => {
                list.push(item.to_string());
            }
            None => {
                tmp.insert(s, vec![item.to_string()]);
            }
        }
    }
    Vec::from_iter(tmp.values().cloned())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn group_anagrams_should_work() {
        let input = vec![
            "abc".to_string(),
            "cba".to_string(),
            "etc".to_string(),
            "cte".to_string(),
        ];
        let result = group_anagrams(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[1].len(), 2);
    }
}
