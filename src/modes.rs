use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    NormalDelete,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Mode::*;
        write!(
            f,
            "{}",
            match self {
                Normal => "Normal",
                NormalDelete => "NormalDelete",
                Insert => "Insert",
                Visual => "Visual",
            }
        )
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
