pub trait CryptocurrencyConfig {
    const ATOMIC_PER_UNIT: u64;
    const SYMBOL: &'static str;
    fn precision() -> usize {
        Self::ATOMIC_PER_UNIT.ilog10() as usize
    }
}

#[derive(Clone, Debug)]
pub struct AtomicUnit<C: CryptocurrencyConfig>(u64, std::marker::PhantomData<C>);

impl<C: CryptocurrencyConfig> AtomicUnit<C> {
    pub fn from_atomic(a: u64) -> Self {
        Self(a, std::marker::PhantomData)
    }

    pub fn as_atomic(&self) -> u64 {
        self.0
    }

    pub fn to_string_trimmed(&self) -> String {
        let whole = self.0 / C::ATOMIC_PER_UNIT;
        let fractional = self.0 % C::ATOMIC_PER_UNIT;

        if fractional == 0 {
            return whole.to_string();
        }

        let frac_str = format!("{:0width$}", fractional, width = C::precision())
            .trim_end_matches('0')
            .to_string();

        format!("{}.{}", whole, frac_str)
    }
}

#[derive(Clone, Debug)]
pub struct XMR;
impl CryptocurrencyConfig for XMR {
    const ATOMIC_PER_UNIT: u64 = 1_000_000_000_000;
    const SYMBOL: &'static str = "XMR";
}

pub type XMRAtomicUnit = AtomicUnit<XMR>;

#[derive(Clone, Debug)]
pub struct LTC;
impl CryptocurrencyConfig for LTC {
    const ATOMIC_PER_UNIT: u64 = 100_000_000;
    const SYMBOL: &'static str = "LTC";
}
pub type LTCAtomicUnit = AtomicUnit<LTC>;
