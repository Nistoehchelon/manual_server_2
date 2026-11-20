use cryptocurrencies_stuff::currencies::{LTC, XMR, XNO};
use financial_stuff::get_by_usd;
use sqlx::{PgConnection, PgPool};

#[derive(
    strum::EnumMessage,
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::AsRefStr,
    strum::EnumIter,
)]
#[strum(serialize_all = "snake_case")]
pub enum ShopService {
    #[strum(
        message = "Allows you to login into your account inside the game client and play in matches, valid for a duration of your choice (up to 12 months at a time). After a subscription expires, you can still sign in on the website, but will not be able to sign in to the game server or play matches until the subscription is renewed."
    )]
    Subscription,
    #[strum(message = "A cosmetic thingie that allows choosing a color for your display username")]
    UsernameColor,
}

#[derive(Debug, Clone)]
pub struct TotalCurrency {
    pub name: String,
    pub amount: String,
}
#[derive(Debug, Clone)]
pub struct ShoppingCart {
    pub total_price_usd: String,
    pub total_prices_in_payment_method_currencies: Vec<TotalCurrency>,
    pub items: Vec<ShoppingCartItem>,
}
#[derive(Debug, Clone)]
pub struct ShoppingCartItem {
    pub usd_price: String,
    pub id: String,
    pub name: String,
    pub data: Vec<ShoppingCartItemDatum>,
}
#[derive(Debug, Clone)]
pub struct ShoppingCartItemDatum {
    pub name: String,
    pub value: String,
}

pub async fn get_item_name(conn: &mut PgConnection, item_id: &str) -> anyhow::Result<String> {
    Ok(sqlx::query!(
        "SELECT name FROM shopping_cart_items WHERE id = $1",
        item_id
    )
    .fetch_one(conn)
    .await?
    .name)
}
pub async fn calculate_service_item_usd_price(
    conn: &mut PgConnection,
    item_id: String,
) -> anyhow::Result<f64> {
    let item_name = get_item_name(conn, &item_id).await?;
    let service = item_name.parse::<ShopService>()?;
    let item_data = get_item_data(conn, &item_id).await?;
    let custom_var = match service {
        ShopService::Subscription => {
            custom_site_variables::CustomSiteVariableName::SubscriptionCostPerMonthUsd
        }
        ShopService::UsernameColor => {
            custom_site_variables::CustomSiteVariableName::UsernameColorCostPerMonthUsd
        }
    };
    let service_cost_usd_per_month =
        custom_site_variables::get_custom_site_variable(&mut *conn, custom_var)
            .await?;
    match service {
        ShopService::Subscription => {
            let months_str = item_data
                .iter()
                .find(|d| d.name == "months")
                .map(|d| d.value.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'months' datum for item {}", item_id))?;
            let service_cost: f64 = service_cost_usd_per_month.parse()?;
            let usd_service_cost = usd_cost(months_str.parse()?, service_cost);
            Ok(usd_service_cost)
        }
        ShopService::UsernameColor => {
            let months_str = item_data
                .iter()
                .find(|d| d.name == "months")
                .map(|d| d.value.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'months' datum for item {}", item_id))?;
            let service_cost: f64 = service_cost_usd_per_month.parse()?;
            let usd_service_cost = usd_cost(months_str.parse()?, service_cost);
            Ok(usd_service_cost)
        }
    }
}

pub async fn get_item_data(
    conn: &mut PgConnection,
    item_id: &String,
) -> Result<Vec<ShoppingCartItemDatum>, anyhow::Error> {
    let data = sqlx::query_as!(
        ShoppingCartItemDatum,
        "SELECT name, value FROM shopping_cart_item_data WHERE shopping_cart_item_id = $1",
        item_id
    )
    .fetch_all(conn)
    .await?;
    Ok(data)
}

fn usd_cost(months: u64, service_cost: f64) -> f64 {
    service_cost * (months as f64)
}

pub async fn calc_price_for_all_currencies(
    db_pool: PgPool,
    total: f64,
) -> anyhow::Result<Vec<TotalCurrency>> {
    let mut tx = db_pool.begin().await?;
    let ltc = get_by_usd::<LTC>(&mut tx, &total.to_string()).await?;
    let xmr = get_by_usd::<XMR>(&mut tx, &total.to_string()).await?;
    let xno = get_by_usd::<XNO>(&mut tx, &total.to_string()).await?;

    let re = vec![
        TotalCurrency {
            name: "LTC".to_owned(),
            amount: ltc.to_string_trimmed(),
        },
        TotalCurrency {
            name: "XMR".to_owned(),
            amount: xmr.to_string_trimmed(),
        },
        TotalCurrency {
            name: "XNO".to_owned(),
            amount: xno.to_string_trimmed(),
        },
    ];
    tx.commit().await?;
    Ok(re)
}
