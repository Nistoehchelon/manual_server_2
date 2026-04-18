use error_structs::InternalAppError;
use html_templating::html_template_structs::{
    self, BaseTemplateData, IndexLoggedInTemplate, IndexNotLoggedInTemplate,
};

use session::Session;
use user::{User, get_user_by_user_id};

use http_stuff::http_response::HttpResponseData;

use crate::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if session.logged_in {
        let user = get_user_by_user_id(db_pool.clone(), session.user_id.clone())
            .await
            .map_err(InternalAppError::new)?;

        return Ok(http_stuff::http_response::ok_html(get_html(
            router_data.base_template_data,
            session.clone(),
            user,
        )));
    }
    Ok(http_stuff::http_response::ok_html(get_html(
        router_data.base_template_data,
        session.clone(),
        User::default(),
    )))
}

fn get_html(
    mut base_template: BaseTemplateData,
    session: Session,
    user: User,
) -> String {
    if session.logged_in {
        base_template.title = "Welcome back!".to_string();

        let index_template = IndexLoggedInTemplate {
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
