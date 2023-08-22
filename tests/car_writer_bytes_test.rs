#[cfg(not(target_arch = "wasm32"))]
mod proptests {
    use cid::Cid;
    use iroh_car::*;
    use multihash::{Code, Multihash, MultihashDigest};
    use proptest::{collection::vec, prelude::any, prop_assert_eq, strategy::Strategy};
    use test_strategy::proptest;

    fn identity_hash_cid() -> impl Strategy<Value = Cid> {
        vec(any::<u8>(), 0..64)
            .prop_map(|hash_bytes| Cid::new_v1(0x55, Multihash::wrap(0x00, &hash_bytes).unwrap()))
    }

    #[proptest(max_shrink_iters = 1_000_000)]
    fn write_returns_bytes_written(
        #[strategy(vec(vec(any::<u8>(), 0..1_000), 0..100))] blocks: Vec<Vec<u8>>,
        #[strategy(vec(identity_hash_cid(), 1..100))] roots: Vec<Cid>,
    ) {
        let (supposedly_written, actually_written) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async move {
                let mut writer = CarWriter::new(CarHeader::new_v1(roots), Vec::new());
                let mut written = 0;
                written += writer.write_header().await.unwrap();
                for block in blocks {
                    let hash = Code::Blake3_256.digest(&block);
                    let cid = Cid::new_v1(0x55, hash);
                    written += writer.write(cid, block).await.unwrap();
                }
                let buffer = writer.finish().await.unwrap();
                (written, buffer.len())
            });

        prop_assert_eq!(supposedly_written, actually_written);
    }
}
