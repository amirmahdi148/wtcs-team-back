use crate::controllers::badges_controller::{PaginationMeta, pagination_metadata};

#[test]
fn pagination_metadata_populates_fields() {
    let meta = pagination_metadata(3, 5, 20);
    assert_eq!(
        meta,
        PaginationMeta {
            current_page: 3,
            total_pages: 5,
            limit: 20,
        },
    );
}

#[test]
fn pagination_metadata_zero_total_pages() {
    let meta = pagination_metadata(1, 0, 10);
    assert_eq!(meta.total_pages, 0);
}
