use askama::Template;

pub fn render<T: Template>(template: T) -> anyhow::Result<String> {
    Ok(template.render()?)
}
