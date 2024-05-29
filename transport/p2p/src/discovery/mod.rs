pub struct Discovery {
    discv5: Discv5,
}

impl Discovery {
    pub async fn new() -> Result<Self> {
        let local_enr_key: CombinedKey = CombinedKey::from_libp2p(local_key);

        let discv5 = Discv5::new(local_enr, local_enr_key).await;
        Self { discv5 }
    }
}
