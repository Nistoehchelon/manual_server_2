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
        let records_count = self.records_count;
        let records_per_page = self.records_per_page;
        let current_page = self.current_page;
        let current_url = self.current_url.clone();
        let page_param = self.page_param.clone();

        let pages_count = records_count.div_ceil(records_per_page);
        let mut pages: Vec<PaginationElement> = vec![];

        if current_page > 1 {
            let mut prev_page_url = current_url.clone();
            prev_page_url
                .query
                .insert(page_param.clone(), format!("{}", current_page - 1));
            pages.push(PaginationElement {
                linked: true,
                link_url: prev_page_url.to_string(),
                name: "<-".to_string(),
            });
        }

        pages.push(PaginationElement {
            linked: false,
            link_url: String::new(),
            name: format!("{}", current_page),
        });

        if current_page < pages_count {
            let mut next_page_url = current_url.clone();
            next_page_url
                .query
                .insert(page_param.clone(), format!("{}", current_page + 1));
            pages.push(PaginationElement {
                linked: true,
                link_url: next_page_url.to_string(),
                name: "->".to_string(),
            });
        }

        if current_page < pages_count {
            pages.push(PaginationElement {
                linked: false,
                link_url: String::new(),
                name: "..".to_string(),
            });
            let mut last_page_url = current_url.clone();
            last_page_url
                .query
                .insert(page_param.clone(), format!("{}", pages_count));
            pages.push(PaginationElement {
                linked: true,
                link_url: last_page_url.to_string(),
                name: format!("{}", pages_count),
            });
        }

        pages
    }
}
