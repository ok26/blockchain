use crate::math::big_int::BigInt;
use std::time::{SystemTime, UNIX_EPOCH};

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut encoded = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        encoded.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        encoded.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if i + 1 < data.len() {
            encoded.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            encoded.push('=');
        }

        if i + 2 < data.len() {
            encoded.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            encoded.push('=');
        }

        i += 3;
    }
    encoded
}

pub fn base64_decode(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 {
        panic!("Invalid Base64 input length");
    }

    let mut decoded = Vec::new();

    let mut char_to_val = [255u8; 256];
    for (i, &c) in BASE64_CHARS.iter().enumerate() {
        char_to_val[c as usize] = i as u8;
    }
    char_to_val[b'=' as usize] = 0;

    let mut i = 0;
    while i < bytes.len() {
        let sextet0 = char_to_val[bytes[i] as usize];
        let sextet1 = char_to_val[bytes[i + 1] as usize];
        let sextet2 = char_to_val[bytes[i + 2] as usize];
        let sextet3 = char_to_val[bytes[i + 3] as usize];

        if sextet0 == 255 || sextet1 == 255 || sextet2 == 255 || sextet3 == 255 {
            panic!("Invalid Base64 character");
        }

        let triple = ((sextet0 as u32) << 18)
            | ((sextet1 as u32) << 12)
            | ((sextet2 as u32) << 6)
            | (sextet3 as u32);

        decoded.push(((triple >> 16) & 0xFF) as u8);

        if bytes[i + 2] != b'=' {
            decoded.push(((triple >> 8) & 0xFF) as u8);
        }
        if bytes[i + 3] != b'=' {
            decoded.push((triple & 0xFF) as u8);
        }

        i += 4;
    }

    decoded
}

fn push_der_length(vec: &mut Vec<u8>, len: usize) {
    if len < 0x80 {
        vec.push(len as u8);
    } else {
        let len_bytes = len.to_be_bytes();
        let len_bytes = len_bytes.iter().skip_while(|b| **b == 0).cloned().collect::<Vec<_>>();
        vec.push(0x80 | (len_bytes.len() as u8));
        vec.extend_from_slice(&len_bytes);
    }
}

fn get_der_length(bytes: &[u8], idx: &mut usize) -> usize {
    let len_byte = bytes[*idx];
    *idx += 1;
    if len_byte & 0x80 == 0 {
        len_byte as usize
    } else {
        let num_bytes = (len_byte & 0x7F) as usize;
        let mut len = 0usize;
        for _ in 0..num_bytes {
            len = (len << 8) | (bytes[*idx] as usize);
            *idx += 1;
        }
        len
    }
}

pub fn der_encode<const T: usize>(fields: &[&BigInt<T>]) -> Vec<u8> {
    let mut der = Vec::new();
    let mut content = Vec::new();

    for field in fields.iter() {
        let bytes = field.to_bytes_be();
        content.push(0x02);
        push_der_length(&mut content, bytes.len());
        content.extend_from_slice(&bytes);
    }

    der.push(0x30);
    push_der_length(&mut der, content.len());
    der.extend_from_slice(&content);
    
    der
}

pub fn der_decode<const T: usize>(der: &[u8]) -> Vec<BigInt<T>> {
    let bytes = der;
    assert_eq!(bytes[0], 0x30);
    let mut idx = 1;
    let content_len = get_der_length(bytes, &mut idx);

    let mut fields = Vec::new();
    while idx < content_len + 1 {
        assert_eq!(bytes[idx], 0x02);
        idx += 1;
        let int_len = get_der_length(bytes, &mut idx);
        let int_bytes = &bytes[idx..idx + int_len];
        idx += int_len;
        fields.push(BigInt::from_bytes_be(int_bytes));
    }

    fields
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}