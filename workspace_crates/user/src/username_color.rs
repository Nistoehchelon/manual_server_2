use sqlx::PgConnection;

use crate::get_user_data;

pub async fn try_get_username_color(
    conn: &mut PgConnection,
    user_id: String,
) -> anyhow::Result<Option<String>> {
    let Some(color_expiry) = get_user_data(conn, &user_id, "username_color_expires_at").await?
    else {
        return Ok(None);
    };
    let Some(color_code) = get_user_data(conn, &user_id, "username_color_code").await? else {
        return Ok(None);
    };
    let color_code = color_code.parse::<csscolorparser::Color>()?;
    let color_code = color_code.name().unwrap_or(&color_code.to_string()).to_string();
    let color_expiry = color_expiry.parse::<u64>()?;
    if color_expiry > time_funcs::utc_now()? {
        Ok(Some(color_code))
    } else {
        Ok(None)
    }
}

pub async fn get_available_username_colors(conn: &mut PgConnection) -> anyhow::Result<Vec<String>> {
    let Ok(content) = custom_site_variables::find_custom_site_variable(
        conn,
        custom_site_variables::CustomSiteVariableName::AllowedUsernameColor,
    )
    .await
    else {
        tracing::error!("Could not get allowed_username_colors");
        return Ok(Vec::new());
    };
    let colors = if let Some(content) = content {
        content
            .lines()
            .map(|line| {
                let line = line.trim();
                line.to_owned()
            })
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };
    Ok(colors)
}

pub async fn color_available(
    conn: &mut PgConnection,
    color: csscolorparser::Color,
) -> anyhow::Result<bool> {
    for raw in get_available_username_colors(conn).await? {
        let parsed = raw.parse::<csscolorparser::Color>()?;
        if parsed == color {
            return Ok(true);
        }
    }

    Ok(false)
}
