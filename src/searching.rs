use std::cmp::min;

fn head(s : &str) -> char {
    s.chars().next().unwrap()
}

fn min3(x : usize, y : usize, z : usize) -> usize {
    let min_xy = min(x, y);
    return min(min_xy, z);
}

pub fn levenshtein_distance(s1 : &str, s2 : &str) -> usize {
    if s1.len() == 0 {
        return s2.len();
    }
    if s2.len() == 0 {
        return s1.len();
    }
    if head(s1) == head(s2) {
        return levenshtein_distance(&s1[1..], &s2[1..]);
    }

    let dis1 = levenshtein_distance(&s1[1..], s2);
    let dis2 = levenshtein_distance(s1, &s2[1..]);
    let dis3 = levenshtein_distance(&s1[1..], &s2[1..]);

    1 + min3(dis1, dis2, dis3)
}