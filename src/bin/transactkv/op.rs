use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub enum Op {
    Read {
        key: usize,
        read: Option<Vec<usize>>,
    },
    Append {
        key: usize,
        value: usize,
    },
}

impl Op {
    fn from_raw(raw: RawOp) -> Option<Self> {
        let RawOp(op, key, value) = raw;
        match (op, value) {
            ("r", Either::Right(None)) => Some(Op::Read { key, read: None }),
            ("append", Either::Left(value)) => Some(Op::Append { key, value }),
            _ => None,
        }
    }

    fn to_raw(&self) -> RawOp {
        match self {
            Op::Read { key, read } => RawOp("r", *key, Either::Right(read.clone())),
            Op::Append { key, value } => RawOp("append", *key, Either::Left(*value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RawOp<'a>(&'a str, usize, Either<usize, Option<Vec<usize>>>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl Serialize for Op {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_raw().serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Op {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let raw = RawOp::deserialize(deserializer)?;
        Op::from_raw(raw).ok_or_else(|| D::Error::custom("Failed to parse op"))
    }
}
