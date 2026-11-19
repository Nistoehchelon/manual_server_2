use html_templating::html_template_structs::{BaseTemplateData, SubscriptionTemplate};

use user::get_user_by_user_id;

use http_stuff::http_response::{HttpResponseData, redirect_to};

use routing_structs::RouterData;

pub async fn get(
    router_data: RouterData,
) -> anyhow::Result<HttpResponseData> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let user = get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        ?;
    let mut tx = db_pool.begin().await?;
    let user_has_a_subscription = user::subscription::user_has_a_subscription(&mut tx, &user.id)
        .await
        ?;
    tx.commit().await?;
    if user_has_a_subscription {
        return Ok(redirect_to("/".to_string()));
    }

    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    ?;
    let usd_cost = financial_stuff::get_subscription_cost_usd(db_pool.clone())
        .await
        ?;
    let xmr_cost = financial_stuff::convert_usd_price_to_xmr(db_pool.clone(), &usd_cost)
        .await
        ?;
    let ltc_cost = financial_stuff::convert_usd_price_to_ltc(db_pool.clone(), &usd_cost)
        .await
        ?;
    let xno_cost = financial_stuff::convert_usd_price_to_xno(db_pool.clone(), &usd_cost)
        .await
        ?;
    Ok(http_stuff::http_response::ok_html(get_html(
        router_data.base_template_data,
        csrf_token,
        usd_cost.to_string(),
        xmr_cost.to_string_trimmed(),
        ltc_cost.to_string_trimmed(),
        xno_cost.to_string_trimmed(),
    )))
}

fn get_html(
    base_template: BaseTemplateData,
    csrf_token: String,
    usd_cost: String,
    xmr_cost: String,
    ltc_cost: String,
    xno_cost: String,
) -> String {
    let sub_template = SubscriptionTemplate {
        csrf_token,
        main_info: base_template,
        usd_cost,
        xmr_cost,
        ltc_cost,
        xno_cost,
    };
    html_templating::render(sub_template)
}
