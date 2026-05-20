use strum_macros::FromRepr;

#[derive(Debug)]
pub struct Deletion {
    pub resource_type: DeletionResourceType,
    pub resource_id: i64,
}

#[derive(Debug, PartialEq, FromRepr)]
#[repr(u8)]
pub enum DeletionResourceType {
    Folder = 0,
    Member = 1,
    CustomField = 2,
    CustomFieldDataValue = 3,
}