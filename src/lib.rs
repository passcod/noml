#![allow(dead_code)]

#[macro_use] extern crate nom;

use nom::{not_line_ending, multispace};
use std::str;

named!(bare_key <&[u8], &str>,
    map_res!(
        is_a!("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"),
        str::from_utf8
    )
);

named!(escaped_string_content <&[u8], String>,
    map_res!(
        escaped_transform!(
            call!(not_line_ending),
            '\\',
            alt!(
                  tag!("\\") => { |_| &b"\\"[..] }
                | tag!("\"") => { |_| &b"\""[..] }
                | tag!("b")  => { |_| &b"\x00\x08"[..] }
                | tag!("t")  => { |_| &b"\t"[..] }
                | tag!("n")  => { |_| &b"\n"[..] }
                | tag!("f")  => { |_| &b"\x00\x0c"[..] }
                | tag!("r")  => { |_| &b"\r"[..] }
                // TODO: Unicode
            )
        ),
        String::from_utf8
    )
);

named!(basic_string <&[u8], String>,
    chain!(
          tag!("\"")
        ~ content: call!(escaped_string_content)
        ~ tag!("\"")
        , || { content }
    )
);


named!(table <&[u8], &str>,
    chain!(
          tag!("[")
        ~ label: map_res!(
            take_until!("]"),
            str::from_utf8
        )
        ~ tag!("]")
        ~ multispace?
        , || { label }
    )
);

named!(comment <&[u8], &str>,
    chain!(
        tag!("#")
        ~ text: map_res!(
            not_line_ending,
            str::from_utf8
        )
        , || { text }
    )
);

#[cfg(test)]
mod test {
    use nom::IResult::{Done, Error};
    use nom::Err::Position;
    use nom::ErrorKind::IsA;

    #[test]
    fn bare_key() {
        assert_eq!(super::bare_key(b"hello"),
            Done(&b""[..], "hello"));
        assert_eq!(super::bare_key(b"WORLD"),
            Done(&b""[..], "WORLD"));
        assert_eq!(super::bare_key(b"MiXeD"),
            Done(&b""[..], "MiXeD"));
        assert_eq!(super::bare_key(b"12345"),
            Done(&b""[..], "12345"));
        assert_eq!(super::bare_key(b"M1x3D"),
            Done(&b""[..], "M1x3D"));
        assert_eq!(super::bare_key(b"-----"),
            Done(&b""[..], "-----"));
        assert_eq!(super::bare_key(b"_____"),
            Done(&b""[..], "_____"));
        assert_eq!(super::bare_key(b"a-1_B"),
            Done(&b""[..], "a-1_B"));

        assert_eq!(super::bare_key(b"no.dot"),
            Done(&b".dot"[..], "no"));
        assert_eq!(super::bare_key(b"no space"),
            Done(&b" space"[..], "no"));
        assert_eq!(super::bare_key(b"no\xbb\xc2unicode"),
            Done(&b"\xbb\xc2unicode"[..], "no"));
        assert_eq!(super::bare_key(b"no\"quote"),
            Done(&b"\"quote"[..], "no"));

        assert_eq!(super::bare_key(b"."),
            Error(Position(IsA, &b"."[..])));
    }

    #[test]
    fn escaped_string_content() {
        assert_eq!(super::escaped_string_content(b"abcde"),
            Done(&b""[..], String::from("abcde")));

        assert_eq!(super::escaped_string_content(b"ab\\\\de"),
            Done(&b""[..], String::from("ab\\de")));
        assert_eq!(super::escaped_string_content(b"ab\\\"de"),
            Done(&b""[..], String::from("ab\"de")));
        assert_eq!(super::escaped_string_content(b"ab\\bde"),
            Done(&b""[..], String::from("ab\x00\x08de")));
        assert_eq!(super::escaped_string_content(b"ab\\tde"),
            Done(&b""[..], String::from("ab\tde")));
        assert_eq!(super::escaped_string_content(b"ab\\nde"),
            Done(&b""[..], String::from("ab\nde")));
        assert_eq!(super::escaped_string_content(b"ab\\fde"),
            Done(&b""[..], String::from("ab\x00\x0cde")));
        assert_eq!(super::escaped_string_content(b"ab\\rde"),
            Done(&b""[..], String::from("ab\rde")));
    }

    #[test]
    fn top_table() {
        assert_eq!(super::table(b"[table]"),
            Done(&b""[..], "table"));
    }

    #[test]
    fn line_comment() {
        assert_eq!(super::comment(b"#a comment "),
            Done(&b""[..], "a comment "));
        assert_eq!(super::comment(b"#a comment"),
            Done(&b""[..], "a comment"));
        assert_eq!(super::comment(b"# a comment"),
            Done(&b""[..], " a comment"));
        assert_eq!(super::comment(b"# a comment\r\n"),
            Done(&b"\r\n"[..], " a comment"));
        assert_eq!(super::comment(b"# a comment\r"),
            Done(&b"\r"[..], " a comment"));
        assert_eq!(super::comment(b"# a comment\n"),
            Done(&b"\n"[..], " a comment"));
    }
}
