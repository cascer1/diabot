#[derive(Debug, PartialEq, poise::ChoiceParameter)]
pub enum GlucoseUnit {
    #[name = "mg/dL"]
    Mgdl,
    #[name = "mmol/L"]
    Mmol,
}
