use html_templating::{
    html_template_structs::{
        AllowedUsernameColorCodesTemplate, BaseTemplateData, ShopMainPageTemplate, UsernameColorTemplate, ViewCartTemplate,
    }, render,
};
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params, http_response::{HttpResponseData, ok_html, redirect_to},
};
use id_generator_unique_in_db::Table::{ShoppingCartItemData, ShoppingCartItems};
use payment_system_structs::PaymentMethod;
use routing_structs::RouterData;
use session::{add_session_notification_error, add_session_notification_success};
use shop::{
    ShopService, ShoppingCart, ShoppingCartItem,
    calc_price_for_all_currencies, calculate_service_item_usd_price, get_item_data,
};
use sqlx::PgPool;
use strum::{EnumMessage, IntoEnumIterator};
use user::get_user_by_user_id;

use crate::{not_found::not_found, subscription};

pub async fn main_page(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let services: Vec<(String, String)> = ShopService::iter()
        .map(|s| {
            (
                s.to_string(),
                s.get_message().unwrap_or_default().to_string(),
            )
        })
        .collect();
    let tpl = ShopMainPageTemplate {
        main_info: router_data.base_template_data.clone(),
        services,
    };

    let html = render(tpl);
    Ok(ok_html(html))
}

pub async fn clear_cart(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let user = get_user_by_user_id(
        router_data.db_pool.clone(),
        router_data.session.user_id.clone(),
    )
    .await?;
    let cart_id =
        possibly_create_and_get_user_cart_id(router_data.db_pool.clone(), user.id.clone()).await?;
    let result = sqlx::query!(
        "DELETE FROM shopping_cart_items WHERE shopping_cart_id = $1",
        cart_id
    )
    .execute(&router_data.db_pool)
    .await?;

    add_session_notification_success(
        router_data.db_pool.clone(),
        router_data.session.id.clone(),
        format!(
            "Cart cleared successfully, removed {} items",
            result.rows_affected()
        ),
    )
    .await;
    Ok(redirect_to("/shop/cart".to_owned()))
}

pub async fn delete_cart_item(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let item_id = router_data
        .http_request_data
        .path
        .strip_prefix("/shop/cart/delete_item/")
        .ok_or_else(|| anyhow::anyhow!("Could not extract item_id from path"))?;
    let result = sqlx::query!("DELETE FROM shopping_cart_items WHERE id = $1", item_id)
        .execute(&router_data.db_pool)
        .await?;
    match result.rows_affected() {
        0 => {
            add_session_notification_error(
                router_data.db_pool.clone(),
                router_data.session.id.clone(),
                "Could not find the item".to_owned(),
            )
            .await;
        }
        _ => {
            add_session_notification_success(
                router_data.db_pool.clone(),
                router_data.session.id.clone(),
                "Item removed successfully".to_owned(),
            )
            .await;
        }
    }
    Ok(redirect_to("/shop/cart".to_owned()))
}
pub async fn view_cart(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let user = get_user_by_user_id(
        router_data.db_pool.clone(),
        router_data.session.user_id.clone(),
    )
    .await?;
    let cart_id =
        possibly_create_and_get_user_cart_id(router_data.db_pool.clone(), user.id.clone()).await?;
    let cart_items = sqlx::query!(
        "SELECT id, name FROM shopping_cart_items WHERE shopping_cart_id = $1",
        cart_id
    )
    .fetch_all(&router_data.db_pool)
    .await?;

    let mut items = Vec::with_capacity(cart_items.len());
    let mut total_price_usd = 0.0;
    for item in cart_items {
        let mut conn = router_data.db_pool.clone().acquire().await?;
        let data = get_item_data(&mut conn, &item.id).await?;

        let cost = calculate_service_item_usd_price(&mut conn, item.id.clone()).await?;
        total_price_usd += cost;
        items.push(ShoppingCartItem {
            usd_price: cost.to_string(),
            id: item.id,
            name: item.name,
            data,
        });
    }
    let total_price_in_payment_method_currencies =
        calc_price_for_all_currencies(router_data.db_pool.clone(), total_price_usd).await?;
    let cart = ShoppingCart {
        items,
        total_prices_in_payment_method_currencies: total_price_in_payment_method_currencies,
        total_price_usd: total_price_usd.to_string(),
    };
    let tpl = ViewCartTemplate {
        main_info: router_data.base_template_data.clone(),
        cart,
        payment_currencies: PaymentMethod::iter().collect(),
    };
    let html = render(tpl);
    Ok(ok_html(html))
}

pub async fn get_items(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let path = router_data.http_request_data.path.clone();
    let Some(service) = path.strip_prefix("/shop/") else {
        tracing::debug!("Could not strip prefix");
        return crate::not_found::not_found(router_data);
    };
    let service: ShopService = match service.parse() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("{e:#?}");
            return not_found(router_data);
        }
    };
    match service {
        ShopService::Subscription => subscription::get(router_data).await,
        ShopService::UsernameColor => get_color(router_data).await,
    }
}
pub async fn get_color(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let _user = get_user_by_user_id(db_pool.clone(), session.user_id.clone()).await?;

    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await?;
    let usd_cost = financial_stuff::get_username_color_cost_per_month_usd(db_pool.clone()).await?;
    let xmr_cost = financial_stuff::convert_usd_price_to_xmr(db_pool.clone(), &usd_cost).await?;
    let ltc_cost = financial_stuff::convert_usd_price_to_ltc(db_pool.clone(), &usd_cost).await?;
    let xno_cost = financial_stuff::convert_usd_price_to_xno(db_pool.clone(), &usd_cost).await?;
    Ok(http_stuff::http_response::ok_html(get_html(
        router_data.base_template_data,
        csrf_token,
        usd_cost.to_string(),
        xmr_cost.to_string_trimmed(),
        ltc_cost.to_string_trimmed(),
        xno_cost.to_string_trimmed(),
    )))
}

pub async fn add_to_cart(
    router_data: RouterData,
) -> anyhow::Result<http_stuff::http_response::HttpResponseData> {
    let user = get_user_by_user_id(
        router_data.db_pool.clone(),
        router_data.session.user_id.clone(),
    )
    .await?;

    let user_cart_id =
        possibly_create_and_get_user_cart_id(router_data.db_pool.clone(), user.id.clone()).await?;

    let body = router_data.http_request_data.clone().body_bytes;
    let params =
        parse_body_into_string_form_input_params(body).map_err(|e| anyhow::anyhow!("{e:#?}"))?;

    let service: ShopService = params
        .get("service")
        .ok_or_else(|| anyhow::anyhow!("Service param not found"))?
        .parse()?;

    let common_tuple = match common(
        router_data.clone(),
        user_cart_id,
        params.clone(),
        service.clone(),
    )
    .await
    {
        Ok(s) => s,
        Err(e) => match e {
            CommonError::MonthsSpecifiedAreBeyondAllowedRange
            | CommonError::ItemAlreadyInUserCart => {
                return Ok(redirect_to("/shop/cart".to_owned()));
            }

            CommonError::PropagateError { message } => {
                return Err(anyhow::anyhow!("{message}"));
            }
        },
    };
    let (months, mut tx, shopping_cart_item_id) = common_tuple;
    match service {
        ShopService::Subscription => {
            let item_data_id =
                id_generator_unique_in_db::generate_unique_id_for(&mut tx, ShoppingCartItemData)
                    .await?;
            sqlx::query!("INSERT INTO shopping_cart_item_data (id, shopping_cart_item_id, name, value) VALUES($1, $2, $3, $4)", item_data_id, shopping_cart_item_id, "months", months.to_string()).execute(&mut *tx).await?;
            tx.commit().await?;

            return Ok(redirect_to("/shop?added".to_owned()));
        }
        ShopService::UsernameColor => {
            let raw_color_code = params
                .get("color_code")
                .ok_or_else(|| anyhow::anyhow!("Color param not found"))?;
            let color_code = match raw_color_code.parse::<csscolorparser::Color>() {
                Ok(cc) => {
                    let conn = &mut *tx;
                    if !user::username_color::get_allowed_username_colors(conn)
                        .await?
                        .contains(&cc.to_string())
                    {
                        add_session_notification_error(
                            router_data.db_pool.clone(),
                            router_data.session.id.clone(),
                            format!(
                                "Color code {} is not on the list of allowed username colors",
                                cc
                            ),
                        )
                        .await;
                        return Ok(redirect_to("/shop/username_color".to_owned()));
                    }
                    cc
                }
                Err(e) => {
                    add_session_notification_error(
                        router_data.db_pool.clone(),
                        router_data.session.id.clone(),
                        format!("Could not parse color {raw_color_code}: {e}"),
                    )
                    .await;
                    return Ok(redirect_to("/shop/username_color".to_owned()));
                }
            };

            let item_data_id =
                id_generator_unique_in_db::generate_unique_id_for(&mut tx, ShoppingCartItemData)
                    .await?;
            sqlx::query!("INSERT INTO shopping_cart_item_data (id, shopping_cart_item_id, name, value) VALUES($1, $2, $3, $4)", item_data_id, shopping_cart_item_id, "color_code", color_code.to_string()).execute(&mut *tx).await?;
            let item_data_id =
                id_generator_unique_in_db::generate_unique_id_for(&mut tx, ShoppingCartItemData)
                    .await?;
            sqlx::query!("INSERT INTO shopping_cart_item_data (id, shopping_cart_item_id, name, value) VALUES($1, $2, $3, $4)", item_data_id, shopping_cart_item_id, "months", months.to_string()).execute(&mut *tx).await?;

            tx.commit().await?;
        }
    }

    add_session_notification_success(
        router_data.db_pool.clone(),
        router_data.session.id.clone(),
        format!("Items {service} added to cart"),
    )
    .await;
    Ok(redirect_to("/shop".to_owned()))
}

pub enum CommonError {
    MonthsSpecifiedAreBeyondAllowedRange,
    ItemAlreadyInUserCart,
    PropagateError { message: String },
}
async fn common(
    router_data: RouterData,
    user_cart_id: String,
    params: std::collections::HashMap<String, String>,
    service: ShopService,
) -> Result<(u64, sqlx::Transaction<'static, sqlx::Postgres>, String), CommonError> {
    let months = params
        .get("months")
        .ok_or_else(|| CommonError::PropagateError {
            message: "No months found in params".to_owned(),
        })?
        .parse::<u64>()
        .map_err(|e| CommonError::PropagateError {
            message: format!("Months parse error: {e:#?}"),
        })?;
    if !(0..=12).contains(&months) {
        add_session_notification_error(
            router_data.db_pool.clone(),
            router_data.session.id.clone(),
            format!("Invalid months {months}, must be between (inclusive) 1 and 12"),
        )
        .await;
        return Err(CommonError::MonthsSpecifiedAreBeyondAllowedRange);
    }
    let existing = sqlx::query!(
        "SELECT id FROM shopping_cart_items WHERE shopping_cart_id = $1 AND name = $2",
        user_cart_id,
        service.to_string()
    )
    .fetch_optional(&router_data.db_pool)
    .await
    .map_err(|e| CommonError::PropagateError {
        message: format!("{e:#?}"),
    })?;
    if existing.is_some() {
        add_session_notification_error(
            router_data.db_pool.clone(),
            router_data.session.id.clone(),
            "This item already exists in your  cart".to_owned(),
        )
        .await;
        return Err(CommonError::ItemAlreadyInUserCart);
    }
    let mut tx =
        router_data
            .db_pool
            .clone()
            .begin()
            .await
            .map_err(|e| CommonError::PropagateError {
                message: format!("{e:#?}"),
            })?;
    let shopping_cart_item_id =
        id_generator_unique_in_db::generate_unique_id_for(&mut tx, ShoppingCartItems)
            .await
            .map_err(|e| CommonError::PropagateError {
                message: format!("{e:#?}"),
            })?;
    sqlx::query!(
        "INSERT INTO shopping_cart_items (id, shopping_cart_id, name) VALUES($1, $2, $3)",
        shopping_cart_item_id,
        user_cart_id,
        service.to_string()
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| CommonError::PropagateError {
        message: format!("{e:#?}"),
    })?;
    Ok((months, tx, shopping_cart_item_id))
}
fn get_html(
    base_template: BaseTemplateData,
    csrf_token: String,
    usd_cost: String,
    xmr_cost: String,
    ltc_cost: String,
    xno_cost: String,
) -> String {
    let sub_template = UsernameColorTemplate {
        csrf_token,
        main_info: base_template,
        usd_cost,
        xmr_cost,
        ltc_cost,
        xno_cost,
    };
    html_templating::render(sub_template)
}

pub async fn possibly_create_and_get_user_cart_id(
    db_pool: PgPool,
    user_id: String,
) -> anyhow::Result<String> {
    let cart_option = sqlx::query!(
        "SELECT id FROM user_shopping_carts WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&db_pool)
    .await?;
    match cart_option {
        Some(id) => Ok(id.id),
        None => {
            let mut tx = db_pool.begin().await?;
            let id = id_generator_unique_in_db::generate_unique_id_for(
                &mut tx,
                id_generator_unique_in_db::Table::UserShoppingCarts,
            )
            .await?;
            sqlx::query!(
                "INSERT INTO user_shopping_carts (user_id, id) VALUES($1, $2)",
                user_id,
                id
            )
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(id)
        }
    }
}

pub async fn get_allowed_username_color_codes(router_data: RouterData) -> anyhow::Result<HttpResponseData> { 
    let mut conn = router_data.db_pool.acquire().await?;
    let colors = user::username_color::get_allowed_username_colors(&mut conn).await?;
    let template = AllowedUsernameColorCodesTemplate { main_info: router_data.base_template_data, colors };
    let html = render(template);
    Ok(ok_html(html))
}