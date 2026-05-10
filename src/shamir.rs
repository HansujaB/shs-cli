use crate::errors::ShsError;
use rand::RngCore;

const PRIME:u64=257;

// modular arithmetic functions

pub fn mod_add(a:u64, b:u64, p:u64)-> u64{
    ((a%p)+(b%p))%p;
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
        if(exp%2==1){
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

