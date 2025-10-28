use std::fmt::Formatter;

#[derive(Debug, PartialEq, poise::ChoiceParameter)]
pub enum EstimationUnit {
    #[name = "average blood glucose"]
    glucose,
    #[name = "HbA1c"]
    a1c,
    #[name = "fructosamine"]
    fructosamine,
}

impl std::fmt::Display for EstimationUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            EstimationUnit::glucose => write!(f, "glucose"),
            EstimationUnit::a1c => write!(f, "a1c"),
            EstimationUnit::fructosamine => write!(f, "fructosamine"),
        }
    }
}
