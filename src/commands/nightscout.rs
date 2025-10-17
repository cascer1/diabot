use crate::serenity::CreateEmbed;
use crate::util::colors::{ERROR, INFO, WARNING};
use crate::util::nightscout::client::NightscoutClient;
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude::{Color, CreateEmbedFooter, Timestamp};
use poise::CreateReply;
use tracing::{error, warn};
use crate::util::nightscout::v1_models;
use crate::util::nightscout::v1_models::CombinedNightscout;
use crate::util::nightscout::v2_models::BgNowPlugin;

#[poise::command(
    slash_command,
    description_localized("en-US", "Get the most recent info from any Nightscout site")
)]
pub async fn nightscout(
    ctx: Context<'_>,
    #[description = "Nightscout url"] url: String,
) -> Result<(), Error> {
    let client = NightscoutClient::new_unauthed(&url)?;

    let embed = CreateEmbed::default().description("*Fetching data...*").color(INFO);
    let reply_handle = ctx.send(CreateReply::default().embed(embed)).await?;

    let author_avatar = ctx.author_member().await.map(|m| m.face()).unwrap_or(ctx.author().face());
    let ns_data = match client.fetch_combined().await {
        Ok(data) => data,
        Err(e) => {
            error!(%url, error = ?e, "Error fetching Nightscout data");
            let embed = if e.is_sensitive() {
                CreateEmbed::default()
                    .title("Error fetching data")
                    .description("See follow-up response for more details")
                    .color(ERROR)
            } else {
                CreateEmbed::default()
                    .title("Error fetching data")
                    .description(e.to_string())
                    .color(ERROR)
            };

            reply_handle.edit(ctx, CreateReply::default().embed(embed)).await?;

            if e.is_sensitive() {
                let detailed_embed = CreateEmbed::default()
                    .title("Error details")
                    .description(e.to_string())
                    .color(ERROR);

                ctx.send(CreateReply::default().embed(detailed_embed).ephemeral(true)).await?;
            }
            return Ok(());
        }
    };

    let response_embed = match build_response(author_avatar, &ns_data) {
        Ok(embed) => embed,
        Err(e) => {
            warn!(error = %e, user = %ctx.author().id, "Failed to build response embed");

            let embed = CreateEmbed::default()
                .title("Missing or invalid data")
                .description(e)
                .color(WARNING);

            reply_handle.edit(ctx, CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    reply_handle.edit(ctx, CreateReply::default().embed(response_embed)).await?;

    Ok(())
}

pub fn build_response(avatar_url: String, ns_data: &CombinedNightscout) -> Result<CreateEmbed, String> {
    let settings = &ns_data.status;
    let props = &ns_data.properties;

    let bgnow = props.bgnow.as_ref().ok_or("BG data is missing")?;
    let delta = props.delta.as_ref().ok_or("BG delta is missing")?;

    let mgdl = format!(
        "{} ({})",
        bgnow.last.to_mgdl().as_numeric_string(),
        delta.mgdl.to_mgdl().as_delta_string(),
    );
    let mmol = format!(
        "{} ({})",
        bgnow.last.to_mmol().as_numeric_string(),
        delta.mgdl.to_mmol().as_delta_string(),
    );

    let mut embed = CreateEmbed::default()
        .title(settings.custom_title.clone())
        .field("mmol/L", mmol,true)
        .field("mg/dL", mgdl, true)
        .thumbnail(avatar_url);

    if let Some(direction) = &props.direction {
        embed = embed.field("trend", &direction.label, true);
    }

    if let Some(iob) = &props.iob
        && iob.iob != 0.0
    {
        embed = embed.field("iob", format!("{:.2}", iob.iob), true);
    }

    if let Some(cob) = &props.cob
        && cob.cob != 0.0
    {
        embed = embed.field("cob", format!("{:.0}", cob.cob), true);
    }

    embed = set_response_color(settings, bgnow, embed);

    let timestamp = Timestamp::from_millis(bgnow.mills)
        .map_err(|_| "Invalid timestamp from BG data")?;
    embed = embed.timestamp(timestamp)
        .footer(
            CreateEmbedFooter::new("measured")
                .icon_url("https://github.com/nightscout/cgm-remote-monitor/raw/master/static/images/large.png")
        );

    let fifteen_min_ago = Utc::now() - chrono::Duration::minutes(15);
    if bgnow.mills < fifteen_min_ago.timestamp_millis() {
        embed = embed.description("**BG data is more than 15 minutes old**");
    }

    Ok(embed)
}

fn set_response_color(settings: &v1_models::Status, bgnow: &BgNowPlugin, embed: CreateEmbed) -> CreateEmbed {
    let glucose = bgnow.last.as_mgdl_value();
    let bg_high = settings.bg_high;
    let bg_target_top = settings.bg_target_top;
    let bg_target_bottom = settings.bg_target_bottom;
    let bg_low = settings.bg_low;

    let color = if glucose >= bg_high || glucose <= bg_low {
        Color::from_rgb(255, 0, 0) // red
    } else if glucose >= bg_target_top && glucose < bg_high || glucose > bg_low && glucose <= bg_target_bottom {
        Color::from_rgb(255, 200, 0) // yellow
    } else {
        Color::from_rgb(0, 255, 0) // green
    };

    embed.color(color)
}
