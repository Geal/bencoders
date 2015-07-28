#[macro_use]
extern crate nom;

use nom::digit;
use std::str;

use std::collections::HashMap;
use std::iter::FromIterator;


#[derive(Debug, Clone)]
pub enum Bencode {
    Bytes(Vec<u8>),
    Int(i64),
    List(Vec<Bencode>),
    Dict(HashMap<Vec<u8>, Bencode>),
}

named!(bytes<(Vec<u8>)>,
    chain!(
        l: number ~
        tag!(":") ~
        b: take!(l),
        || { b.to_vec() }
    )
);

named!(number<i64>,
    chain!(
        s: opt!(tag!("-")) ~
        b: digit,
        || {
            let val = str::from_utf8(b).unwrap().parse::<i64>().unwrap();
            match s {
                Some(_) => -val,
                None => val,
            }
        }
    )
);

named!(int<i64>,
    chain!(
        tag!("i") ~
        i: number ~
        tag!("e"),
        || { i }
    )
);

named!(list<(Vec<Bencode>)>,
    chain!(
        tag!("l") ~
        x: many0!(decode) ~
        tag!("e"),
        || { x }
    )
);

named!(dict<(HashMap<Vec<u8>, Bencode>)>,
    chain!(
        tag!("d") ~
        x: many0!(
            pair!(bytes, decode)
        ) ~
        tag!("e"),
        || { HashMap::from_iter(x) }
    )
);

named!(pub decode<Bencode>,
    alt!(
        chain!(b: bytes, || { Bencode::Bytes(b) }) |
        chain!(i: int, || { Bencode::Int(i) }) |
        chain!(l: list, || { Bencode::List(l) }) |
        chain!(d: dict, || { Bencode::Dict(d) })
    )
);

#[cfg(test)]
mod tests {
    use super::{decode, Bencode};
    use nom::IResult;

    #[test]
    fn decode_bytes() {
        let val = b"4:spam";
        let b = match decode(val) {
            IResult::Done(_, Bencode::Bytes(b)) => b,
            _ => panic!("Wrong"),
        };
        assert_eq!("spam".as_bytes(), &b[..]);
    }

    #[test]
    fn decode_zero_bytes() {
        let val = b"0:";
        let b = match decode(val) {
            IResult::Done(_, Bencode::Bytes(b)) => b,
            _ => panic!("Wrong"),
        };
        assert_eq!("".as_bytes(), &b[..]);
    }

    #[test]
    fn decode_positive_int() {
        let val = b"i123e";
        let i = match decode(val) {
            IResult::Done(_, Bencode::Int(i)) => i,
            _ => panic!("Wrong"),
        };
        assert_eq!(123, i);
    }

    #[test]
    fn decode_negative_int() {
        let val = b"i-123e";
        let i = match decode(val) {
            IResult::Done(_, Bencode::Int(i)) => i,
            _ => panic!("Wrong"),
        };
        assert_eq!(-123, i);
    }

    #[test]
    fn decode_zero_int() {
        let val = b"i0e";
        let i = match decode(val) {
            IResult::Done(_, Bencode::Int(i)) => i,
            _ => panic!("Wrong"),
        };
        assert_eq!(0, i);
    }

    #[test]
    fn decode_list() {
        let val = b"li12345ei0ee";
        let l = match decode(val) {
            IResult::Done(_, Bencode::List(l)) => l,
            _ => panic!("Wrong"),
        };
        assert_eq!(2, l.len());
        match l[0] {
            Bencode::Int(i) => assert_eq!(12345, i),
            _ => panic!("Wrong"),
        };
    }

    #[test]
    fn decode_empty_list() {
        let val = b"le";
        let l = match decode(val) {
            IResult::Done(_, Bencode::List(l)) => l,
            _ => panic!("Wrong"),
        };
        assert_eq!(0, l.len());
    }

    #[test]
    fn decode_dict() {
        let val = b"d5:counti12345ee";
        let d = match decode(val) {
            IResult::Done(_, Bencode::Dict(d)) => d,
            _ => panic!("Wrong"),
        };
        assert_eq!(1, d.len());
        assert_eq!(true, d.contains_key(&b"count".to_vec()));
        match d.get(&b"count".to_vec()).unwrap() {
            &Bencode::Int(i) => assert_eq!(12345, i),
            _ => panic!("Wrong"),
        }
    }

    #[test]
    fn decode_empty_dict() {
        let val = b"de";
        let d = match decode(val) {
            IResult::Done(_, Bencode::Dict(d)) => d,
            _ => panic!("Wrong"),
        };
        assert_eq!(0, d.len());
    }
}
