use url::Url;

pub struct PaginationElement {
    pub linked: bool,
    pub link_url: String,
    pub name: String,
}

pub struct Pagination {
    pub records_count: u64,
    pub records_per_page: u64,
    pub current_page: u64,
    pub current_url: Url,
    pub page_param: String,
}

impl Pagination {
    pub fn get_pagination_elements(&self) -> Vec<PaginationElement> {
        let records_count = self.records_count.clone();
        let records_per_page = self.records_per_page.clone();
        let current_page = self.current_page.clone();
        let current_url = self.current_url.clone();
        let page_param = self.page_param.clone();

        let pages_count = (records_count + records_per_page - 1) / records_per_page;
        let mut pages: Vec<PaginationElement> = vec![];

        if current_page - 1 > 1 {
            let mut prev_page_url = current_url.clone();
            prev_page_url
                .query
                .insert(page_param.clone(), format!("{}", current_page - 1));
            pages.push(PaginationElement {
                linked: true,
                link_url: format!("{}", prev_page_url.to_string()),
                name: format!("<-"),
            });
        }

        pages.push(PaginationElement {
            linked: false,
            link_url: format!(""),
            name: format!("{}", current_page),
        });

        if current_page + 1 <= pages_count {
            let mut next_page_url = current_url.clone();
            next_page_url
                .query
                .insert(page_param.clone(), format!("{}", current_page + 1));
            pages.push(PaginationElement {
                linked: true,
                link_url: format!("{}", next_page_url.to_string()),
                name: format!("->"),
            });
        }

        pages
    }
}
