pub trait DataPreparer{
    fn prepare(&self, data: &Vec<u8>)->DataPrepareResult; }
pub struct DataPrepareResult{
    pub data_remains: Vec<u8>,
    pub service_data: Vec<u8> }