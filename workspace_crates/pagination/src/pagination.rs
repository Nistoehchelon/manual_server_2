pub struct Page {
    pub linked: bool,
    pub link_url: String,
    pub name: String,
}
pub async fn get_pages(records_count: u64, records_per_page: u64, _current_page: u64,_base_url: String, ) -> Vec<Page> {
    let _pages_count = (records_count + records_per_page - 1) / records_per_page;
    let mut pages: Vec<Page> = vec![];
    pages.push(Page {
        linked: false,
        link_url: format!("/test"),
        name: format!("<"),
    });

    pages 
}