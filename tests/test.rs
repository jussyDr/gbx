use gbx::{Block, Item, Map};
use paste::paste;
use std::io::BufReader;

fn test_read_block(block_id: u32, hash: &str) {
    let url = format!("https://item.exchange/item/download/{block_id}");
    let file = test_util::fetch_file(&url, hash, env!("CARGO_TARGET_TMPDIR")).unwrap();
    let reader = BufReader::new(file);

    Block::read_from(reader).unwrap();
}

macro_rules! test_read_block {
    ($id:literal, $hash:literal) => {
        paste! {
            #[test]
            fn [<read_block_ $id>]() {
                test_read_block($id, $hash);
            }
        }
    };
}

test_read_block!(5899, "Of3Y0ecMmelzrYhrqseEjkq16yvUXsPTS5WZGS_5Bdc");
test_read_block!(19019, "OLMBYCB4V32uQKAP39qdV8pEY8j1Mmd36cYfzoZWQIs");
test_read_block!(43839, "DquHIX6wG-cgI_x68oqH81sdhAZCC7IX9YJ_4qlA6Gw");
test_read_block!(44867, "U6JKwKAv62gS_KLHuJpaSc0Ri5mHvbitGodiceC-5qI");

fn test_read_item(item_id: u32, hash: &str) {
    let url = format!("https://item.exchange/item/download/{item_id}");
    let file = test_util::fetch_file(&url, hash, env!("CARGO_TARGET_TMPDIR")).unwrap();
    let reader = BufReader::new(file);

    Item::read_from(reader).unwrap();
}

macro_rules! test_read_item {
    ($id:literal, $hash:literal) => {
        paste! {
            #[test]
            fn [<read_item_ $id>]() {
                test_read_item($id, $hash);
            }
        }
    };
}

test_read_item!(5933, "vvSSNoLARSsF0XkoVuOtjUs6qu2-JGPel84zKUA6pVQ");
test_read_item!(21172, "iaLvppeLVEDLEo8XRAz2kORTE6aBTRACHyM0JESqc3s");
test_read_item!(26427, "-56XYR9Zubtctt_EBzKJ-NCrERE-JVeSGetzPH0URNY");
test_read_item!(42887, "-grSbU1L361FVqv0U03cARSNmUH5KnSgFLEvg1zTMGM");
test_read_item!(44357, "1KCsxGkqUe0AqfNUEM1BCBAKE_MaQZjeDHZ9olZJthM");
test_read_item!(45331, "nBM1Y3OlRxlH5kvfTALN0zZXNenGElSlOLB82RX2g_s");

fn test_read_map(map_id: u32, hash: &str) {
    let url = format!("https://trackmania.exchange/maps/download/{map_id}");
    let file = test_util::fetch_file(&url, hash, env!("CARGO_TARGET_TMPDIR")).unwrap();
    let reader = BufReader::new(file);

    Map::read_from(reader).unwrap();
}

macro_rules! test_read_map {
    ($id:literal, $hash:literal) => {
        paste! {
            #[test]
            fn [<read_map_ $id>]() {
                test_read_map($id, $hash);
            }
        }
    };
}

test_read_map!(1795, "vyYfYNInhD4KdTtKBx9InfspQblAPqN7llCGq-q40mI");
test_read_map!(1984, "_eAiK0BClvjSW9hp9j1RgYuKCVl250e3QXreRsa1440");
test_read_map!(19387, "AynCKf2FlfrPeQHGXpaRpvEn02iEaqedY6I9NYxMZSI");
test_read_map!(46951, "RZY3fk02zYm2UrTSQx3xdl1omZ7GC1c5rY4CzD5WvXs");
test_read_map!(59807, "zZIa_CIe3s7-NFvT9kfcQh0w4wugFTRCpF-3zgDVoQM");
test_read_map!(65123, "zTIsc39uOpH6DZel064l9vMqod207-3oWX2U8TCybhs");
test_read_map!(81283, "kchS0VpCEqL23krWoZt5Dm1I6by_kwy384HgRNRHT8k");

#[test]
fn write_read_default_map() {
    let map = Map::default();
    let mut buf = vec![];
    map.write_to(&mut buf).unwrap();
    Map::read_from(buf.as_slice()).unwrap();
}
