
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
    #[strum(
        message = "A costumetic thingie that allows choosing a color for your display username"
    )]
    UsernameColor,
}




#[derive(Debug, Clone)]
pub struct ShoppingCart {
    pub items: Vec<ShoppingCartItem>,
}
#[derive(Debug, Clone)]
pub struct ShoppingCartItem {
    pub name: String,
    pub data: Vec<ShoppingCartItemDatum>,
}
#[derive(Debug, Clone)]
pub struct ShoppingCartItemDatum {
    pub name: String,
    pub value: String,
}