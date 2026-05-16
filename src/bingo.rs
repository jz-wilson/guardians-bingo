use std::collections::HashSet;

pub const BINGO_LINES: [[usize; 5]; 12] = [
    [0, 1, 2, 3, 4],
    [5, 6, 7, 8, 9],
    [10, 11, 12, 13, 14],
    [15, 16, 17, 18, 19],
    [20, 21, 22, 23, 24],
    [0, 5, 10, 15, 20],
    [1, 6, 11, 16, 21],
    [2, 7, 12, 17, 22],
    [3, 8, 13, 18, 23],
    [4, 9, 14, 19, 24],
    [0, 6, 12, 18, 24],
    [4, 8, 12, 16, 20],
];

pub fn winning_line(marked: &HashSet<usize>) -> Option<[usize; 5]> {
    BINGO_LINES
        .iter()
        .find(|line| line.iter().all(|i| marked.contains(i)))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_row_wins() {
        let marked: HashSet<usize> = [0, 1, 2, 3, 4].iter().copied().collect();
        assert!(winning_line(&marked).is_some());
    }

    #[test]
    fn main_diagonal_wins() {
        let marked: HashSet<usize> = [0, 6, 12, 18, 24].iter().copied().collect();
        assert!(winning_line(&marked).is_some());
    }

    #[test]
    fn anti_diagonal_wins() {
        let marked: HashSet<usize> = [4, 8, 12, 16, 20].iter().copied().collect();
        assert!(winning_line(&marked).is_some());
    }

    #[test]
    fn no_win_incomplete() {
        let marked: HashSet<usize> = [0, 1, 2, 3].iter().copied().collect();
        assert!(winning_line(&marked).is_none());
    }
}
