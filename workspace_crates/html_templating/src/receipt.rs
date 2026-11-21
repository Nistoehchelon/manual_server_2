use askama::Template;

use crate::html_template_structs::BaseTemplateData;

pub struct ReceiptItemDataForTemplate {
    pub name: String,
    pub value: String,
}
pub struct ReceiptItemForTemplate {
    pub name: String,
    pub data: Vec<ReceiptItemDataForTemplate>,
}

#[derive(Template)]
#[template(path = "shop/receipt_shape.html")]
pub struct ReceiptShellTemplate {
    pub main_info: BaseTemplateData,
    pub items: Vec<ReceiptItemForTemplate>,
    pub receipt_id: String,
    pub root_website_url: String,
    pub datetime: String,
    pub buyer_current_username: String,
    pub buyer_user_id: String,
    pub seller_name: String,
    pub seller_pubkey_url: String,
    pub currency_name: String,
    pub currency_ticker: String,
    pub currency_total_amount: String,
    pub reference_currency: String,
    pub reference_currency_total_amount: String,
    pub reference_currency_per_currency: String,
}
