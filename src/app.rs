use crate::route::Route;

pub enum Field {
    LatestBlocks,
    LatestTransactions,
    Bottom,
    Details,
}

impl Field {
    pub fn get_index(&self) -> usize {
        match self {
            Self::LatestBlocks => 0,
            Self::LatestTransactions => 1,
            Self::Bottom => 2,
            Self::Details => 3,
        }
    }
}

#[derive(Debug)]
pub enum SidebarCategory {
    LatestBlocks,
    LatestTransactions,
    Bottom,
}

impl From<usize> for SidebarCategory {
    fn from(id: usize) -> Self {
        if id == 0 {
            Self::LatestBlocks
        } else if id == 1 {
            Self::LatestTransactions
        } else if id == 2 {
            Self::Bottom
        } else {
            panic!()
        }
    }
}

pub struct App<'a> {
    pub route: Route,
    pub sidebar_items: Vec<&'a str>,
    pub focus: usize,
    pub details_about: Option<SidebarCategory>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            route: Route::Home,
            sidebar_items: vec!["Latest Blocks", "Latest Transactions", "Bottom"],
            focus: 0,
            details_about: None,
        }
    }

    pub fn set(&mut self, focus: usize) {
        if focus < 3 {
            self.details_about = None;
        } else if focus == 3 {
            self.details_about = Some(SidebarCategory::from(self.focus));
        }
        self.focus = focus;
    }

    /*
    pub fn next(&mut self) {
        self.focus = (self.focus + 1) % self.sidebar_items.len();
    }


    pub fn previous(&mut self) {
        if self.focus > 0 {
            self.focus -= 1;
        } else {
            self.focus = self.sidebar_items.len() - 1;
        }
    }
    */
}