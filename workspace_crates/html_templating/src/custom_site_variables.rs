use askama::Template;
use custom_site_variables::CustomSiteVariableEntry;

use crate::html_template_structs::BaseTemplateData;

#[derive(Template)]
#[template(path = "edit_custom_site_variables.html")]
pub struct EditCustomSiteVariablesTemplate {
    pub main_info: BaseTemplateData,
    pub variables: Vec<CustomSiteVariableEntry>,   
    pub csrf_token: String, 
}


