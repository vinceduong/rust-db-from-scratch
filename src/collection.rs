use crate::{
    collection_file::CollectionFile,
    collection_indexer::{index_collection_id, IdToPageMap},
    document::{Document, HasId},
};

struct Collection<T: Document> {
    id_to_page_map: IdToPageMap<T>,
    collection_file: CollectionFile<T>,
}

impl<T: Document> Collection<T> {
    fn new(name: &str) -> Collection<T> {
        let collection_file = CollectionFile::new(name, "./collection").unwrap();
        let collection_id_idx = index_collection_id(&collection_file).unwrap();

        Collection {
            id_to_page_map: collection_id_idx,
            collection_file,
        }
    }

    fn find_by_id(&self, id: <T as HasId>::Id) -> Option<T> {
        let page_number = self.id_to_page_map.get(&id)?;

        let page = self.collection_file.read_page(*page_number).ok()?;

        page.find_document(id)
    }
}
