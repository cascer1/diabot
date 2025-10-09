use crate::conversions::glucose::ParsedGlucoseResult;
use crate::serenity::CreateEmbed;
use crate::util::colors::{ERROR, INFO, WARNING};
use crate::{Context, Error};
use poise::ChoiceParameter;

#[derive(Debug, poise::ChoiceParameter)]
pub enum GlucoseUnit {
    #[name = "mg/dL"]
    Mgdl,
    #[name = "mmol/L"]
    Mmol,
}

/// Converts blood glucose units (mg/dL <> mmol/L).
#[poise::command(
    slash_command,
    description_localized("en-US", "Converts blood glucose units between mmol/L and mg/dL")
)]
pub async fn convert(
    ctx: Context<'_>,
    #[description = "The value to convert (e.g. 5.7mmol, 100 mg, 40)"] glucose: String,
    #[description = "Blood glucose unit (mmol/L, mg/dL)"] unit: Option<GlucoseUnit>,
) -> Result<(), Error> {
    let reply = match ParsedGlucoseResult::parse(&glucose, unit.map(|u| u.name())) {
        Ok(glucose_value) => match glucose_value {
            ParsedGlucoseResult::Known(bg) => {
                let embed = CreateEmbed::default().color(INFO).description(format!(
                    "{} is {}",
                    bg,
                    bg.convert()
                ));
                poise::CreateReply::default().embed(embed)
            }

            ParsedGlucoseResult::Ambiguous {
                original,
                as_mgdl,
                as_mmol,
            } => {
                let description = format!(
                    "*I'm not sure if **{original}** is mmol/L or mg/dL, so I'll give you both.*\n\
                        - {} is **{}**\n\
                        - {} is **{}**",
                    as_mgdl,
                    as_mgdl.convert(),
                    as_mmol,
                    as_mmol.convert(),
                );

                let embed = CreateEmbed::default()
                    .color(WARNING)
                    .description(description);
                poise::CreateReply::default().embed(embed)
            }
        },
        Err(e) => {
            let error_embed = CreateEmbed::default()
                .title("Invalid Input")
                .description(format!(
                    "I couldn't understand your input.\n\n**Reason:** {e}\n\n\
                    Please make sure you're entering a number optionally followed by a unit."
                ))
                .color(ERROR)
                .field(
                    "Examples of valid input",
                    "`/convert 5.7mmol`\n`/convert 100 mgdl`\n`/convert 30`",
                    false,
                );

            poise::CreateReply::default()
                .embed(error_embed)
                .ephemeral(true)
        }
    };

    ctx.send(reply).await?;
    Ok(())
}
