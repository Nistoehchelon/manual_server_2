use error_structs::InternalAppError;
use html_templating::html_template_structs::{
    self, BaseTemplateData, IndexLoggedInTemplate, IndexNotLoggedInTemplate,
};

use session::Session;
use user::{User, get_user_by_user_id};

use http_stuff::http_response::HttpResponseData;

use routing_structs::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if session.logged_in {
        let user = get_user_by_user_id(db_pool.clone(), session.user_id.clone())
            .await
            .map_err(InternalAppError::new)?;
        let mut tx = db_pool.begin().await.map_err(InternalAppError::new)?;
        let user_has_a_subscription = user::subscription::user_has_a_subscription(&mut tx, &user.id).await.map_err(InternalAppError::new)?;
        tx.commit().await.map_err(InternalAppError::new)?;

        return Ok(http_stuff::http_response::ok_html(get_html(
            router_data.base_template_data,
            session.clone(),
            user,
            user_has_a_subscription,
        )));
    }
    Ok(http_stuff::http_response::ok_html(get_html(
        router_data.base_template_data,
        session.clone(),
        User::default(),
        false,
    )))
}

fn get_html(mut base_template: BaseTemplateData, session: Session, user: User, user_has_a_subscription: bool) -> String {
    if session.logged_in {
        base_template.title = "Welcome back!".to_string();

        let index_template = IndexLoggedInTemplate {
            user_has_a_subscription,
            main_info: base_template,
            user: html_template_structs::User {
                username: user.username.clone(),
            },
        };
        html_templating::render(index_template)
    } else {
        base_template.title = "Welcome".to_string();
        let index_template = IndexNotLoggedInTemplate {
            main_info: base_template,
        };
        html_templating::render(index_template)
    }
}
