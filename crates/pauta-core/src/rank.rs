//! Fractional ordering keys (section 5.4, `rank`).
//!
//! Moving a card writes only that card: its new rank is a string that sorts
//! between its new neighbours. Port of David Greenspan's "fractional indexing"
//! midpoint algorithm over base-36 digits, without the integer part.

const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

fn digit(c: u8) -> Option<usize> {
    DIGITS.iter().position(|d| *d == c)
}

/// A rank is valid if it is non-empty, uses only base-36 lowercase digits and
/// does not end in `0` (so there is always room before it).
pub fn is_valid(rank: &str) -> bool {
    !rank.is_empty() && rank.bytes().all(|b| digit(b).is_some()) && !rank.ends_with('0')
}

/// Returns a key strictly between `before` and `after` (either may be absent,
/// meaning "start" / "end" of the column). `None` if the inputs are invalid or
/// out of order, in which case the caller re-spaces the column.
pub fn between(before: Option<&str>, after: Option<&str>) -> Option<String> {
    let a = before.unwrap_or("");
    if !a.is_empty() && !is_valid(a) {
        return None;
    }
    if let Some(b) = after
        && (!is_valid(b) || (!a.is_empty() && a >= b))
    {
        return None;
    }
    Some(midpoint(a.as_bytes(), after.map(str::as_bytes)))
}

fn midpoint(a: &[u8], b: Option<&[u8]>) -> String {
    if let Some(b) = b {
        let mut n = 0;
        while n < b.len() && a.get(n).copied().unwrap_or(b'0') == b[n] {
            n += 1;
        }
        if n > 0 {
            let prefix = String::from_utf8_lossy(&b[..n]).into_owned();
            let rest_a = if n < a.len() { &a[n..] } else { &[] };
            return prefix + &midpoint(rest_a, Some(&b[n..]));
        }
    }
    let da = a.first().and_then(|c| digit(*c)).unwrap_or(0);
    let db = b.map_or(DIGITS.len(), |b| digit(b[0]).unwrap_or(DIGITS.len()));
    if db - da > 1 {
        let mid = (da + db).div_ceil(2);
        return (DIGITS[mid] as char).to_string();
    }
    match b {
        Some(b) if b.len() > 1 => (b[0] as char).to_string(),
        _ => {
            let rest = if a.len() > 1 { &a[1..] } else { &[] };
            (DIGITS[da] as char).to_string() + &midpoint(rest, None)
        }
    }
}

/// `n` evenly spaced keys, used to re-space a column whose ranks are broken.
pub fn spread(n: usize) -> Vec<String> {
    let mut out = Vec::with_capacity(n);
    let mut prev: Option<String> = None;
    for _ in 0..n {
        let next = between(prev.as_deref(), None).expect("appending is always possible");
        out.push(next.clone());
        prev = Some(next);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn basics() {
        assert_eq!(between(None, None).unwrap(), "i");
        let r = between(Some("a3"), Some("b1")).unwrap();
        assert!("a3" < r.as_str() && r.as_str() < "b1", "{r}");
        let r = between(Some("a"), Some("b")).unwrap();
        assert!("a" < r.as_str() && r.as_str() < "b", "{r}");
        assert!(between(Some("b"), Some("a")).is_none());
        assert!(between(Some("A!"), None).is_none());
    }

    proptest! {
        /// Repeated insertions between two neighbours never collide (section 15.1).
        #[test]
        fn repeated_insertions_stay_ordered(choices in proptest::collection::vec(any::<bool>(), 1..200)) {
            let mut lo = between(None, None).unwrap();
            let mut hi = between(Some(&lo), None).unwrap();
            for go_low in choices {
                let mid = between(Some(&lo), Some(&hi)).unwrap();
                prop_assert!(lo < mid && mid < hi, "{lo} < {mid} < {hi}");
                prop_assert!(is_valid(&mid));
                if go_low { hi = mid } else { lo = mid }
            }
        }

        #[test]
        fn prepend_and_append_stay_ordered(n in 1usize..100) {
            let mut first = between(None, None).unwrap();
            for _ in 0..n {
                let k = between(None, Some(&first)).unwrap();
                prop_assert!(k < first && is_valid(&k));
                first = k;
            }
            let keys = spread(n);
            prop_assert!(keys.windows(2).all(|w| w[0] < w[1]));
        }
    }
}
