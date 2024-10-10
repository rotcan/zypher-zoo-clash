use ark_bn254::Fr;
use ark_ec::{AffineRepr, CurveGroup};
use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective, Fq};
use ark_ff::{BigInteger, PrimeField};
use ark_serialize::{Compress, Validate};
use rand_chacha::{
    rand_core::{CryptoRng, RngCore, SeedableRng},
    ChaChaRng,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uzkge::{
    plonk::{constraint_system::TurboCS, indexer::PlonkProof},
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
};
use crate::error::ShuffleError;
use crate::{MaskedCard as Masked, };
use crate::card_maps::CARD_MAPS;
use crate::keygen::{Keypair as CoreKeypair};

pub type ShuffleResult<T> =Result<T,ShuffleError>;

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct MaskedCard(pub String, pub String, pub String, pub String);


#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct Keypair {
    /// 0xHex (U256)
    pub sk: String,
    /// 0xHex (U256)
    pub pk: String,
    /// public key uncompress x, y
    pub pkxy: (String, String),
}


#[derive(Serialize, Deserialize,Debug)]
pub struct MaskedCardWithProof {
    /// MaskedCard
    pub card: MaskedCard,
    /// hex string
    pub proof: String,
}


#[derive(Serialize, Deserialize,Debug)]
pub struct RevealedCardWithProof {
    /// MaskedCard
    pub card: (String, String),
    /// hex string
    pub proof: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct RevealedCardWithSnarkProof {
    pub card: (String, String),
    pub snark_proof: Vec<String>,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct ShuffledCardsWithProof {
    /// MaskedCard
    pub cards: Vec<MaskedCard>,
    /// hex string
    pub proof: String,
}


#[allow(non_snake_case)]
#[derive(Serialize, Deserialize,Debug)]
pub struct BigNumber {
    _isBigNumber: bool,
    _hex: String,
}
 
#[inline(always)]
pub(crate) fn util_error_value<T: Display>(e: T) -> ShuffleError {
    ShuffleError::UtilError(e.to_string())
}


pub fn default_prng() -> impl RngCore + CryptoRng {
    ChaChaRng::from_entropy()
}

pub fn hex_to_scalar<F: PrimeField>(hex: &str) -> ShuffleResult<F> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    if bytes.len() != 32 {
        return Err(ShuffleError::UtilError("Bytes length not 32".to_owned()));
    }
    Ok(F::from_be_bytes_mod_order(&bytes))
}

pub fn scalar_to_hex<F: PrimeField>(scalar: &F, with_start: bool) -> String {
    let bytes = scalar.into_bigint().to_bytes_be();
    let s = hex::encode(&bytes);
    if with_start {
        format!("0x{}", s)
    } else {
        s
    }
}

pub fn hex_to_point<G: CurveGroup>(hex: &str) -> ShuffleResult<G> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    G::deserialize_with_mode(bytes.as_slice(), Compress::Yes, Validate::Yes)
        .map_err(util_error_value)
}

pub fn point_to_hex<G: CurveGroup>(point: &G, with_start: bool) -> String {
    let mut bytes = Vec::new();
    point
        .serialize_with_mode(&mut bytes, Compress::Yes)
        .unwrap();
    let s = hex::encode(&bytes);
    if with_start {
        format!("0x{}", s)
    } else {
        s
    }
}

pub fn point_to_uncompress<F: PrimeField, G: CurveGroup<BaseField = F>>(
    point: &G,
    with_start: bool,
) -> (String, String) {
    let affine = G::Affine::from(*point);
    let (x, y) = affine.xy().unwrap();
    let x_bytes = x.into_bigint().to_bytes_be();
    let y_bytes = y.into_bigint().to_bytes_be();
    let x = hex::encode(&x_bytes);
    let y = hex::encode(&y_bytes);

    if with_start {
        (format!("0x{}", x), format!("0x{}", y))
    } else {
        (x, y)
    }
}

pub fn uncompress_to_point(x_str: &str, y_str: &str) -> ShuffleResult<EdwardsProjective> {
    let x_hex = x_str.trim_start_matches("0x");
    let y_hex = y_str.trim_start_matches("0x");
    let x_bytes = hex::decode(x_hex).map_err(util_error_value)?;
    let y_bytes = hex::decode(y_hex).map_err(util_error_value)?;

    let x = Fq::from_be_bytes_mod_order(&x_bytes);
    let y = Fq::from_be_bytes_mod_order(&y_bytes);
    let affine = EdwardsAffine::new(x, y);

    Ok(affine.into())
}

pub fn shuffle_proof_from_hex(s: &str) -> ShuffleResult<PlonkProof<KZGCommitmentSchemeBN254>> {
    let hex = s.trim_start_matches("0x");
    let bytes = hex::decode(hex).map_err(util_error_value)?;
    PlonkProof::<KZGCommitmentSchemeBN254>::from_bytes_be::<TurboCS<Fr>>(&bytes)
        .map_err(util_error_value)
}

pub fn shuffle_proof_to_hex(proof: &PlonkProof<KZGCommitmentSchemeBN254>) -> String {
    let bytes = proof.to_bytes_be();
    format!("0x{}", hex::encode(bytes))
}


pub fn index_to_point(index: i32) -> EdwardsProjective {
    let y_hex = CARD_MAPS[index as usize].trim_start_matches("0x");
    let y_bytes = hex::decode(y_hex).unwrap();
    let y = Fq::from_be_bytes_mod_order(&y_bytes);

    let affine = EdwardsAffine::get_point_from_y_unchecked(y, true).unwrap();
    affine.into()
}

pub fn point_to_index(point: EdwardsProjective) -> ShuffleResult<i32> {
    let affine = EdwardsAffine::from(point);
    let y_bytes = affine.y.into_bigint().to_bytes_be();
    let bytes = format!("0x{}", hex::encode(&y_bytes));

    if let Some(pos) = CARD_MAPS.iter().position(|y| y == &bytes) {
        Ok(pos as i32)
    } else {
        Err(util_error_value("Point not map to  a card"))
    }
}

pub fn masked_card_serialize(masked: &Masked) -> MaskedCard {
    let (e1_x, e1_y) = point_to_uncompress(&masked.e1, true);
    let (e2_x, e2_y) = point_to_uncompress(&masked.e2, true);
    MaskedCard(e2_x, e2_y, e1_x, e1_y)
}

pub fn masked_card_deserialize(masked: &MaskedCard) -> ShuffleResult<Masked> {
    let e2 = uncompress_to_point(&masked.0, &masked.1)?;
    let e1 = uncompress_to_point(&masked.2, &masked.3)?;
    Ok(Masked { e1, e2 })
}


pub fn generate_key_preset(sk : String, pk: String, x : String,y: String) -> ShuffleResult<Keypair> {
    let mut prng = default_prng();
    let keypair = CoreKeypair::generate(&mut prng);
    let _pkxy = point_to_uncompress(&keypair.public, true);

    let ret = Keypair {
        sk: sk,
        pk: pk,
        pkxy: (x,y),
    };

    Ok(ret)
}