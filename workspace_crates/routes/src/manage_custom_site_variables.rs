use custom_site_variables::{get_all_custom_site_variables, save_custom_site_variables_from_form};
use error_structs::InternalAppError;
use http_stuff::http_response::{self, HttpResponseData, redirect_to};
use routing_structs::RouterData;
use session::add_session_notification_success;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let usr_resl = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    if !session.logged_in
        || !abac::can_edit_site_variables(&db_pool, usr_resl.id)
            .await
            .map_err(InternalAppError::new)?
    {
        session::add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            String::from("Insufficient permissions to access this amazing page"),
        )
        .await;
        return Ok(redirect_to("/".to_string()));
    }

    let mut conn = db_pool.acquire().await.map_err(InternalAppError::new)?;
    let vars = get_all_custom_site_variables(&mut conn)
        .await
        .map_err(InternalAppError::new)?;
    let tpl = html_templating::custom_site_variables::EditCustomSiteVariablesTemplate {
        main_info: router_data.base_template_data.clone(),
        variables: vars,
        csrf_token: csrf_stuff::get_or_create_token(
            db_pool.clone(),
            router_data.session.id,
            csrf_stuff::DesignatedPurpose::Normal,
        )
        .await
        .map_err(InternalAppError::new)?,
    };
    let html = html_templating::render(tpl);

    Ok(http_response::ok_html(html))
}

pub async fn post(
    router_data: RouterData,
) -> anyhow::Result<HttpResponseData, error_structs::InternalAppError> {
    use std::collections::HashMap;

    let form: HashMap<String, String> =
        serde_urlencoded::from_bytes(&router_data.http_request_data.body_bytes)
            .map_err(InternalAppError::new)?;

    let mut tx = router_data.db_pool.begin().await.map_err(InternalAppError::new)?;
    save_custom_site_variables_from_form(&mut tx, &form).await.map_err(InternalAppError::new)?;
    add_session_notification_success(router_data.db_pool.clone(), router_data.session.id.clone(), "Errpr setting/modifying vars".to_owned()).await;
    Ok(redirect_to("/manage_custom_site_variables".to_owned()))

}
