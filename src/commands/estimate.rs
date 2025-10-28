use crate::commands::estimateUnit::EstimationUnit;
use crate::{Context, Error};
use crate::util::colors::{ERROR, INFO, WARNING};

#[poise::command(
slash_command,
description_localized("en-US", "Estimate results between average glucose, HbA1c, and fructosamine")
)]
pub async fn estimate(
    ctx: Context<'_>,
    #[description = "The value to convert from"] value: String,
    #[description = "The unit to convert from"] from_unit: EstimationUnit,
    #[description = "The unit to convert to"] to_unit: Option<EstimationUnit>
) -> Result<(), Error> {
    Ok(())
}
