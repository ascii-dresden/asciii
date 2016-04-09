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

#[cfg(test)]
mod list_move{
    //! when checking items of a list and appending them to an error list,
    //! is it faster to copy or to drain and move?

    use std::path::Path;
    use super::test::Bencher;


    #[bench]
    fn drain(b: &mut Bencher) {
        fn existence<'a>(mut paths:Vec<&'a str>) -> Vec<&'a str>{
            paths.drain(..).collect()
        }
        b.iter(||{ 
            existence(vec![
                     "foo",
                     "bar",
                     "foobar",
                     "foobarbazdeadbeefcafebaberustisfast",
            ])
        });
    }

    #[bench]
    fn copy(b: &mut Bencher) {
        fn existence<'a>(mut paths:&[&'a str]) -> Vec<&'a str>{
            paths.iter().map(|s|s.to_owned()).collect()
        }
        b.iter(||{ 
            existence(&[
                     "foo",
                     "bar",
                     "foobar",
                     "foobarbazdeadbeefcafebaberustisfast",
            ])
        });
    }

}
