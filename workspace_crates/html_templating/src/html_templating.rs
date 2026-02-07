use askama::Template;

pub fn render<T: Template>(template: T) -> String {
    template.render().unwrap()
}
