use crate::excerpt_collection::ExcerptCollection;

pub struct ExcerptCollections {
    collections: Vec<ExcerptCollection>,
    num_selected: usize,
}

impl ExcerptCollections {
    pub fn new(collections: Vec<ExcerptCollection>) -> Self {
        Self {
            collections,
            num_selected: 0,
        }
    }

    pub fn get_selected(&self) -> &ExcerptCollection {
        &self.collections[self.num_selected]
    }

    pub fn select_next(&mut self) {
        self.num_selected = (self.num_selected + 1).min(self.collections.len() - 1)
    }

    pub fn select_previous(&mut self) {
        if self.num_selected == 0 {
            return;
        }
        self.num_selected -= 1;
    }

    pub fn select(&mut self, num: usize) {
        self.num_selected = num;
    }

    pub fn enumerate(&self) -> Box<dyn Iterator<Item = (usize, &ExcerptCollection)> + '_> {
        Box::new(self.collections.iter().enumerate())
    }

    pub fn get_selected_index(&self) -> usize {
        self.num_selected
    }

    pub fn len(&self) -> usize {
        self.collections.len()
    }
}
