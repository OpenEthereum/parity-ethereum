use std::ptr;
use spec::ParamType;
use error::Error;
use token::Token;

pub struct Decoder;

struct DecodeResult {
	token: Token,
	new_offset: usize,
}

fn slice_data(data: Vec<u8>) -> Result<Vec<[u8; 32]>, Error> {
	if data.len() % 32 != 0 {
		return Err(Error::InvalidData);
	}

	let times = data.len() / 32;
	let mut result = vec![];
	for i in 0..times {
		let mut slice = [0u8; 32];
		unsafe {
			ptr::copy(data.as_ptr().offset(32 * i as isize), slice.as_mut_ptr(), 32);
		}
		result.push(slice);
	}
	Ok(result)
}

impl Decoder {
	pub fn decode(types: Vec<ParamType>, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		let slices = try!(slice_data(data));
		let mut tokens = vec![];
		let mut offset = 0;
		for param in types.into_iter() {
			let res = try!(Self::decode_param(param, &slices, offset));
			offset = res.new_offset;
			tokens.push(res.token);
		}
		Ok(tokens)
	}

	fn peek(slices: &Vec<[u8; 32]>, position: usize) -> Result<&[u8; 32], Error> {
		slices.get(position).ok_or(Error::InvalidData)
	}

	fn decode_param(param: ParamType, slices: &Vec<[u8; 32]>, offset: usize) -> Result<DecodeResult, Error> {
		match param {
			ParamType::Address => {
				let slice = try!(Self::peek(slices, offset));
				let mut address = [0u8; 20];
				unsafe {
					ptr::copy(slice.as_ptr().offset(12), address.as_mut_ptr(), 20);
				}

				let result = DecodeResult {
					token: Token::Address(address),
					new_offset: offset + 1,
				};

				Ok(result)
			},
			ParamType::Int => {
				let slice = try!(Self::peek(slices, offset));

				let result = DecodeResult {
					token: Token::Int(slice.clone()),
					new_offset: offset + 1,
				};

				Ok(result)
			},
			ParamType::Uint => {
				let slice = try!(Self::peek(slices, offset));

				let result = DecodeResult {
					token: Token::Uint(slice.clone()),
					new_offset: offset + 1,
				};

				Ok(result)
			},
			ParamType::FixedArray(types) => {
				let mut tokens = vec![];
				let mut new_offset = offset;
				for param in types.into_iter() {
					let res = try!(Self::decode_param(param, &slices, new_offset));
					new_offset = res.new_offset;
					tokens.push(res.token);
				}

				let result = DecodeResult {
					token: Token::FixedArray(tokens),
					new_offset: new_offset,
				};

				Ok(result)
			},
			_ => { 
				unimplemented!()
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use token::Token;
	use spec::ParamType;
	use super::{Decoder};

	#[test]
	fn decode_address() {
		let encoded = "0000000000000000000000001111111111111111111111111111111111111111".from_hex().unwrap();
		let address = Token::Address([0x11u8; 20]);
		let expected = vec![address];
		let decoded = Decoder::decode(vec![ParamType::Address], encoded).unwrap();
		assert_eq!(decoded, expected);	
	}

	#[test]
	fn decode_two_address() {
		let encoded = ("".to_owned() + 
					   "0000000000000000000000001111111111111111111111111111111111111111" +
					   "0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap();
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let expected = vec![address1, address2];
		let decoded = Decoder::decode(vec![ParamType::Address, ParamType::Address], encoded).unwrap();
		assert_eq!(decoded, expected);	
	}

	#[test]
	fn decode_fixed_array_of_addresses() {
		let encoded = ("".to_owned() + 
					   "0000000000000000000000001111111111111111111111111111111111111111" +
					   "0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap();
		let address1 = Token::Address([0x11u8; 20]);
		let address2 = Token::Address([0x22u8; 20]);
		let expected = vec![Token::FixedArray(vec![address1, address2])];
		let decoded = Decoder::decode(vec![ParamType::FixedArray(vec![ParamType::Address, ParamType::Address])], encoded).unwrap();
		assert_eq!(decoded, expected);	
	}

	#[test]
	fn decode_uint() {
		let encoded = "1111111111111111111111111111111111111111111111111111111111111111".from_hex().unwrap();
		let uint = Token::Uint([0x11u8; 32]);
		let expected = vec![uint];
		let decoded = Decoder::decode(vec![ParamType::Uint], encoded).unwrap();
		assert_eq!(decoded, expected);	
	}

	#[test]
	fn decode_int() {
		let encoded = "1111111111111111111111111111111111111111111111111111111111111111".from_hex().unwrap();
		let int = Token::Int([0x11u8; 32]);
		let expected = vec![int];
		let decoded = Decoder::decode(vec![ParamType::Int], encoded).unwrap();
		assert_eq!(decoded, expected);	
	}
}

