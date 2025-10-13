use crate::serenity::CreateEmbed;
use crate::util::colors::{ERROR, INFO};
use crate::util::nightscout::client::NightscoutClient;
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude::{Color, CreateEmbedFooter, Timestamp};
use poise::CreateReply;
use tracing::{debug, warn};
use crate::util::nightscout::v1_models::CombinedNightscout;

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
    let ns_data = client.fetch_combined().await;
    if let Err(e) = &ns_data {
        warn!("Error fetching data: {:?}", e);
        let embed = CreateEmbed::default()
            .title("Error fetching data")
            .description("See follow-up response for more details")
            .color(ERROR);
        let descriptive_embed = CreateEmbed::default()
            .title("Error details")
            .description(format!("```rust\n{:?}\n```", e))
            .color(ERROR);
        reply_handle.edit(ctx, CreateReply::default().embed(embed)).await?;
        ctx.send(CreateReply::default().embed(descriptive_embed).ephemeral(true)).await?;
        return Ok(());
    }

    let embed = build_response(author_avatar, &ns_data.unwrap());
    reply_handle.edit(ctx, CreateReply::default().embed(embed)).await?;

    Ok(())
}

pub fn build_response(avatar_url: String, ns_data: &CombinedNightscout) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let settings = &ns_data.status;
    let props = &ns_data.properties;

    let mgdl = format!("{} ({})", props.bgnow.last.to_mgdl().as_numeric_string(), props.delta.mgdl.to_mgdl().as_delta_string());
    let mmol = format!("{} ({})", props.bgnow.last.to_mmol().as_numeric_string(), props.delta.mgdl.to_mmol().as_delta_string());
    embed = embed.title(settings.custom_title.clone())
        .field("mmol/L", mmol,true)
        .field("mg/dL", mgdl, true)
        .field("trend", &props.direction.label, true);

    if let Some(iob_plugin) = &props.iob
        && iob_plugin.iob != 0.0
    {
        embed = embed.field("iob", format!("{:.2}", iob_plugin.iob), true);
    }

    if let Some(cob_plugin) = &props.cob
        && cob_plugin.cob != 0.0
    {
        embed = embed.field("cob", format!("{:.0}", cob_plugin.cob), true);
    }

    embed = set_response_color(ns_data, embed);

    embed = embed.thumbnail(avatar_url);

    embed = embed.timestamp(Timestamp::from_millis(props.bgnow.mills).unwrap())
        .footer(CreateEmbedFooter::new("measured").icon_url("https://github.com/nightscout/cgm-remote-monitor/raw/master/static/images/large.png"));

    let fifteen_min_ago = Utc::now() - chrono::Duration::minutes(15);
    if props.bgnow.mills < fifteen_min_ago.timestamp_millis() {
        embed = embed.description("**BG data is more than 15 minutes old**");
    }

    embed
}

fn set_response_color(ns_data: &CombinedNightscout, mut embed: CreateEmbed) -> CreateEmbed {
    let settings = &ns_data.status;
    let props = &ns_data.properties;
    let glucose = props.bgnow.last.as_mgdl_value();
    let bg_high = settings.bg_high;
    let bg_target_top = settings.bg_target_top;
    let bg_target_bottom = settings.bg_target_bottom;
    let bg_low = settings.bg_low;

    if glucose >= bg_high || glucose <= bg_low {
        embed = embed.color(Color::from_rgb(255, 0, 0));
    } else if glucose >= bg_target_top && glucose < bg_high || glucose > bg_low && glucose <= bg_target_bottom {
        embed = embed.color(Color::from_rgb(255, 200, 0));
    } else {
        embed = embed.color(Color::from_rgb(0, 255, 0));
    }

    embed
}
