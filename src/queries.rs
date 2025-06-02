use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddMessageQuery{
    pub username : String,
    pub msg : String,
    pub gif_id : String,
    pub owner_key : String
}


#[derive(Deserialize)]
pub struct DeleteMessageQuery{
    pub index : usize,
    pub owner_key : String
}


