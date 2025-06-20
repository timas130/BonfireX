use std::net::IpAddr;

pub struct RawUserContext {
    pub id: i64,
    pub ip: IpAddr,
    pub user_agent: String,
}
