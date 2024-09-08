use crate::fm_io::data_offset::ChunkOffset;

#[derive(Debug)]
pub struct FMTable {
    pub table_name: String,
}

#[derive(Debug)]
pub struct FMTableRef {
    pub table_name: ChunkOffset,
}

#[derive(Debug)]
pub struct Script {

}

#[derive(Debug)]
pub struct FMRelation {

}

#[derive(Debug)]
pub struct FMScript {

}
