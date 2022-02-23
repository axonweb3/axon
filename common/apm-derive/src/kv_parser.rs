use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct KeyValue(HashMap<String, String>);

impl KeyValue {
    pub fn inner(self) -> HashMap<String, String> {
        self.0
    }
}

impl FromStr for KeyValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.trim().to_string();

        if !s.starts_with('{') {
            return Err("Must start with '{'".to_string());
        }

        if !s.ends_with('}') {
            return Err("Must end with '}'".to_string());
        }

        s.remove(s.len() - 1);
        s.remove(0);

        let k_strip: &[_] = &[' ', '\n'];
        let v_strip: &[_] = &[' ', '\n', ':'];

        let map = s
            .as_str()
            .split(',')
            .map(|p| {
                let p = p.trim();
                let (k, v) = p.split_at(p.find(':').unwrap());
                (
                    k.trim_matches(k_strip).to_string(),
                    v.trim_matches(v_strip).to_string(),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(KeyValue(map))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use super::*;

    #[test]
    fn test_basic() {
        let test_str = "{next_number: next_number}";
        let res = KeyValue::from_str(test_str).unwrap().inner();

        assert_eq!(res.len(), 1);
        assert_eq!(
            res.get_key_value("next_number").unwrap(),
            (
                "next_number".to_string().borrow(),
                "next_number".to_string().borrow()
            )
        );
    }

    #[test]
    fn test_complex() {
        let test_str = "{current_number: current_number, txs_len:
            commit.content.inner.block.ordered_tx_hashes.len()}";

        let res = KeyValue::from_str(test_str).unwrap().inner();
        assert_eq!(res.len(), 2);
        assert_eq!(
            res.get("txs_len").unwrap(),
            "commit.content.inner.block.ordered_tx_hashes.len()"
        );
    }
}
