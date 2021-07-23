use crate::excerpt_collection::ExcerptCollection;

pub struct ExcerptCollections {
    collections: Vec<ExcerptCollection>,
    num_selected: usize,
}

impl ExcerptCollections {
    pub fn new(collections: Vec<ExcerptCollection>) -> Self {
        Self {
            collections,
            num_selected: 0
        }
    }

    pub fn get_selected(&self) -> &ExcerptCollection {
        &self.collections[self.num_selected]
    }
}
