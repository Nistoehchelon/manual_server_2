use error_structs::InternalAppError;
use html_templating::html_template_structs::{
    BaseTemplateData, SubscriptionTemplate,
};

use user::get_user_by_user_id;

use http_stuff::http_response::{HttpResponseData, redirect_to};

use crate::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let user = get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let mut tx = db_pool.begin().await.map_err(InternalAppError::new)?;
    let user_has_a_subscription = user::subscription::user_has_a_subscription(&mut tx, &user.id)
        .await
        .map_err(InternalAppError::new)?;
    tx.commit().await.map_err(InternalAppError::new)?;
    if user_has_a_subscription {
        return Ok(redirect_to("/".to_string()));
    }

    Ok(http_stuff::http_response::ok_html(get_html(
        router_data.base_template_data,
    )))
}

fn get_html(
    base_template: BaseTemplateData,
) -> String {

    let sub_template = SubscriptionTemplate {
       
        main_info: base_template,
        usd_cost: "1".to_string(),
        xmr_cost: "0.0025".to_string(),
        ltc_cost: "0.017".to_string(),
    };
    html_templating::render(sub_template)
}
