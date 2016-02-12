#![feature(test)]
#[cfg(test)]
extern crate test;


#[cfg(test)]
mod path_split{

    use std::path::Path;
    use super::test::Bencher;

    static PATH: &'static str = "event/dates/1";

    fn split_1<'a>(key:&str) -> Vec<&str>{
        Path::new(key).iter().map(|s|s.to_str().unwrap_or("")).collect::<Vec<&str>>()
    }

    fn split_2<'a>(key:&str) -> Vec<&str>{
        Path::new(key).iter().filter_map(|s|s.to_str()).collect::<Vec<&str>>()
    }

    fn split_3<'a>(key:&str) -> Vec<&str>{
        key.split('/') .collect::<Vec<&str>>()
    }

    // TODO implement a nom based version

    #[bench]
    fn bench_split_1(b: &mut Bencher) {
        b.iter(||{ split_1(PATH); });
    }

    #[bench]
    fn bench_split_2(b: &mut Bencher) {
        b.iter(||{ split_2(PATH); });
    }

    #[bench]
    fn bench_split_3(b: &mut Bencher) {
        b.iter(||{ split_3(PATH); });
    }

}
