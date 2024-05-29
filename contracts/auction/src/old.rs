
pub struct AuctionsIndexes<'a> {
    pub id: MultiIndex<'a, u64, Auction, String>,
    pub offer_denom: MultiIndex<'a, String, Auction, String>,
    pub ask_denom: MultiIndex<'a, String, Auction, String>,
}

impl<'a> IndexList<Auction> for AuctionsIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Auction>> + '_> {
        let v: Vec<&dyn Index<Auction>> = vec![&self.id, &self.offer_denom, &self.ask_denom];
        Box::new(v.into_iter())
    }
}

pub fn auctions<'a>() -> IndexedMap<'a, &'a str, Auction, AuctionsIndexes<'a>> {
    let indexes = AuctionsIndexes {
        id: MultiIndex::new(|_pk, d: &Auction| d.id.clone(), "auctions", "auctions__id"),
        offer_denom: MultiIndex::new(|_pk, d: &Auction| d.offer_coin.denom.clone(), "auctions", "auctions__offer_denom"),
        ask_denom: MultiIndex::new(|_pk, d: &Auction| d.ask_denom.clone(), "auctions", "auctions__ask_denom"),
    };

    IndexedMap::new("tokens", indexes)
}