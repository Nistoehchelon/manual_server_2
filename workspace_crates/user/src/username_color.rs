use sqlx::PgConnection;

use crate::get_user_data;

pub async fn try_get_username_color(conn: &mut PgConnection, user_id: String) -> anyhow::Result<Option<csscolorparser::Color>> {
    let Some(color_expiry) = get_user_data(conn, &user_id, "username_color_expires_at").await? else { return Ok(None) };
    let Some(color_code) = get_user_data(conn, &user_id, "username_color_code").await? else { return Ok(None) };
    let color_code = color_code.parse::<csscolorparser::Color>()?;
    let color_expiry = color_expiry.parse::<u64>()?;
    if color_expiry > time_funcs::utc_now()? {
        Ok(Some(color_code))
    } else {
        Ok(None)
    }
}