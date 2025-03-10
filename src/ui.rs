use ratatui::layout::{Constraint, Constraint::Length};
use unicode_width::UnicodeWidthStr;

pub fn table_column_constraints<S: AsRef<str>>(
    header_strings: &[S],
    row_strings: &[Vec<S>],
) -> Vec<Constraint> {
    row_strings
        .iter()
        .map(|r| r.iter().map(|s| s.as_ref().width()).collect::<Vec<usize>>())
        .fold(
            header_strings
                .iter()
                .map(|s| s.as_ref().width())
                .collect::<Vec<usize>>(),
            |acc, x| {
                acc.iter()
                    .zip(x)
                    .map(|(a, b)| *a.max(&b))
                    .collect::<Vec<usize>>()
            },
        )
        .into_iter()
        .map(|l| Length(l as u16))
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(
        vec!["a", "bb", "ccc"],
        vec![],
        vec![Length(1), Length(2), Length(3)],
    )]
    #[case(
        vec!["name", "age"],
        vec![
            vec!["alice", "20"],
            vec!["bob", "30"],
        ],
        vec![Length(5), Length(3)],
    )]
    #[case(
        vec!["a", "barbar"],
        vec![
            vec!["foo", "b"],
        ],
        vec![Length(3), Length(6)],
    )]
    fn test_table_column_constraints(
        #[case] header_strings: Vec<&str>,
        #[case] row_strings: Vec<Vec<&str>>,
        #[case] expected: Vec<Constraint>,
    ) {
        assert_eq!(
            table_column_constraints(&header_strings, &row_strings),
            expected
        );
    }
}
