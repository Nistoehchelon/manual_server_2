use askama::Template;


pub fn render<T: Template>(template: T) -> String {
    return template.render().unwrap();
}