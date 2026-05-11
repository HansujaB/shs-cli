use crate::errors::ShsError;
use rand::Rng;

const PRIME:u64=257;

// define share struct
// single share produced by splitting a secret
#[derive(Debug, Clone)]
pub struct Share{
    // x-coordinate of this share (1-based, max 255)
    pub index:u8,
    // y-values: one per byte of the secret, i.e. f_k(index) for each byte k
    pub data: Vec<u16>,
}

// modular arithmetic functions

pub fn mod_add(a:u64, b:u64, p:u64)-> u64{
    ((a%p)+(b%p))%p
}

pub fn mod_sub(a:u64, b:u64, p:u64) -> u64{
    let (a,b)= (a%p,b%p);
    if a>=b {(a-b)%p} else {(p+a-b)%p}
}

pub fn mod_mul(a:u64,b:u64,p:u64)->u64{
    ((a%p)*(b%p))%p
}

// for fast exponentiation
pub fn mod_pow(mut base:u64, mut exp:u64, p:u64) -> u64{
    let mut result= 1u64;
    base %= p;
    while exp>0 {
        if exp%2==1{
            result= mod_mul(result,base,p);
        }
        exp /=2;
        base = mod_mul(base,base,p);
    }
    result
}

// modular inverse using fermats theroem
pub fn mod_inv(a:u64, p:u64)-> Result<u64,ShsError> {
    if a%p==0 {
        return Err(ShsError::ReconstructionFailed{
            reason:"cannot invert 0".into(),
        })
    };
    Ok(mod_pow(a,p-2,p))
}

// polynomial helper functions

// Evaluate polynomial at `x`. coeffs[0] is the constant term (the secret byte)
fn eval_polynomial(coeffs: &[u64], x: u64, p: u64) -> u64 {
    let mut result = 0u64;
    let mut power = 1u64; // x^0 = 1
    for &c in coeffs {
        result = mod_add(result, mod_mul(c, power, p), p);
        power = mod_mul(power, x, p);
    }
    result
}


// public API
// Split `secret` into `num_shares` shares with the given `threshold`.
pub fn split(secret: &[u8], threshold: usize, num_shares: usize) -> Result<Vec<Share>, ShsError> {
    if secret.is_empty() {
        return Err(ShsError::EmptySecret);
    }
    if threshold == 0 || threshold > num_shares || num_shares > 255 {
        return Err(ShsError::InvalidThreshold { threshold, num_shares });
    }
    let mut rng = rand::thread_rng();
    // Pre-allocate shares
    let mut shares: Vec<Share> = (1..=num_shares)
        .map(|i| Share { index: i as u8, data: Vec::with_capacity(secret.len()) })
        .collect();
    // For each byte, build a random polynomial and evaluate at each index
    for &byte in secret {
        let mut coeffs = vec![byte as u64];
        for _ in 1..threshold {
            coeffs.push(rng.gen_range(0..PRIME));
        }
        for share in &mut shares {
            let y = eval_polynomial(&coeffs, share.index as u64, PRIME);
            share.data.push(y as u16);
        }
    }
    Ok(shares)
}
// Reconstruct the secret from at least `threshold` shares via Lagrange interpolation.
pub fn reconstruct(shares: &[Share], threshold: usize) -> Result<Vec<u8>, ShsError> {
    if shares.len() < threshold {
        return Err(ShsError::InsufficientShares {
            threshold,
            provided: shares.len(),
        });
    }
    let secret_len = shares[0].data.len();
    for s in shares {
        if s.data.len() != secret_len {
            return Err(ShsError::ReconstructionFailed {
                reason: "shares have mismatched lengths".into(),
            });
        }
    }
    let mut secret = Vec::with_capacity(secret_len);
    for pos in 0..secret_len {
        let mut value = 0u64;
        for i in 0..shares.len() {
            let xi = shares[i].index as u64;
            let yi = shares[i].data[pos] as u64;
            // Lagrange basis L_i(0) = ∏_{j≠i} (0 - xj) / (xi - xj)
            let mut num = 1u64;
            let mut den = 1u64;
            for j in 0..shares.len() {
                if i == j { continue; }
                let xj = shares[j].index as u64;
                num = mod_mul(num, mod_sub(0, xj, PRIME), PRIME);
                den = mod_mul(den, mod_sub(xi, xj, PRIME), PRIME);
            }
            let inv = mod_inv(den, PRIME)?;
            let basis = mod_mul(num, inv, PRIME);
            value = mod_add(value, mod_mul(yi, basis, PRIME), PRIME);
        }
        if value > 255 {
            return Err(ShsError::ReconstructionFailed {
                reason: format!("value {} is not a valid byte", value),
            });
        }
        secret.push(value as u8);
    }
    Ok(secret)
}

// Tests 

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let secret = b"hello world";
        let shares = split(secret, 3, 5).unwrap();
        let recovered = reconstruct(&shares[0..3], 3).unwrap();
        assert_eq!(recovered, secret);
    }
    #[test]
    fn any_t_of_n_works() {
        let secret = b"secret!";
        let shares = split(secret, 3, 5).unwrap();
        // Different combos of 3
        for combo in [[0,1,2], [0,2,4], [1,3,4], [2,3,4]] {
            let subset: Vec<Share> = combo.iter().map(|&i| shares[i].clone()).collect();
            assert_eq!(reconstruct(&subset, 3).unwrap(), secret);
        }
    }
    #[test]
    fn insufficient_shares_fail() {
        let shares = split(b"test", 3, 5).unwrap();
        assert!(reconstruct(&shares[0..2], 3).is_err());
    }
    #[test]
    fn threshold_one() {
        let secret = b"trivial";
        let shares = split(secret, 1, 5).unwrap();
        for s in &shares {
            assert_eq!(reconstruct(&[s.clone()], 1).unwrap(), secret);
        }
    }
    #[test]
    fn threshold_equals_n() {
        let secret = b"all required";
        let shares = split(secret, 5, 5).unwrap();
        assert_eq!(reconstruct(&shares, 5).unwrap(), secret);
        assert!(reconstruct(&shares[0..4], 5).is_err());
    }
    #[test]
    fn single_byte() {
        let secret = &[42u8];
        let shares = split(secret, 2, 3).unwrap();
        assert_eq!(reconstruct(&shares[0..2], 2).unwrap(), secret);
    }
    #[test]
    fn large_secret_1kb() {
        let secret: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let shares = split(&secret, 3, 5).unwrap();
        assert_eq!(reconstruct(&shares[0..3], 3).unwrap(), secret);
    }
    #[test]
    fn empty_secret_fails() {
        assert!(split(b"", 2, 3).is_err());
    }
    #[test]
    fn invalid_threshold() {
        assert!(split(b"x", 0, 3).is_err());
        assert!(split(b"x", 4, 3).is_err());
    }
    #[test]
    fn eval_polynomial_basic() {
        // f(x) = 3 + 2x + x²  →  f(2) = 3 + 4 + 4 = 11
        assert_eq!(eval_polynomial(&[3, 2, 1], 2, 257), 11);
    }
    #[test]
    fn mod_inv_works() {
        let inv = mod_inv(3, 257).unwrap();
        assert_eq!(mod_mul(3, inv, 257), 1);
    }
}
