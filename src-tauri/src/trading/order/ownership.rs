//! Confirmed fill ownership, separate from broker-wide observed holdings.
//!
//! Amounts are integer native units (KRW / USD cents). This ledger contains no
//! credentials, quotes, speculative signal state, or protection high-water marks.
use crate::{
    broker::{BrokerMarket, BrokerScope},
    storage::order_store::OrderSide,
};
use anyhow::{ensure, Context, Result};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{
    collections::{BTreeMap, HashSet},
    fmt,
};

const VERSION: u32 = 1;
const MAX_POSITIONS: usize = 10_000;
const MAX_WATERMARKS: usize = 50_000;
const MAX_ID_BYTES: usize = 256;
const MAX_METADATA_BYTES: usize = 64 * 1024;
const MAX_METADATA_DEPTH: usize = 64;

/// Exact execution identity. None is an explicit legacy profile absence; an
/// unknown account is never sufficient evidence to establish ownership.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OwnershipIdentity {
    pub scope: BrokerScope,
    #[serde(deserialize_with = "explicit_optional_profile")]
    pub profile_id: Option<String>,
    pub is_paper: bool,
}

impl fmt::Debug for OwnershipIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnershipIdentity")
            .field("broker_id", &self.scope.broker_id)
            .field("account_configured", &self.scope.account_id.is_some())
            .field("profile_configured", &self.profile_id.is_some())
            .field("is_paper", &self.is_paper)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OwnershipOwner {
    Strategy(String),
    Manual,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OwnershipKey {
    pub identity: OwnershipIdentity,
    pub owner: OwnershipOwner,
    pub market: BrokerMarket,
    pub symbol: String,
}

impl fmt::Debug for OwnershipKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owner = match self.owner {
            OwnershipOwner::Strategy(_) => "strategy",
            OwnershipOwner::Manual => "manual",
        };
        formatter
            .debug_struct("OwnershipKey")
            .field("identity", &self.identity)
            .field("owner", &owner)
            .field("market", &self.market)
            .field("symbol", &self.symbol)
            .finish()
    }
}

/// Positive held quantity always has a known positive remaining cost basis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OwnershipPosition {
    pub quantity: u64,
    pub cost_notional: u128,
}

/// Cumulative amounts, never the last execution's price times the new delta.
/// Unknown provider quantities or consideration must remain None and are rejected.
#[derive(Clone)]
pub(crate) struct OwnershipFill {
    pub order_record_id: String,
    pub key: OwnershipKey,
    pub side: OrderSide,
    pub ordered_quantity: u64,
    pub cumulative_quantity: Option<u64>,
    pub cumulative_notional: Option<u128>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OwnershipApplied {
    pub delta_quantity: u64,
    pub delta_notional: u128,
    pub released_cost: u128,
    pub realized_pnl: i128,
    pub position: OwnershipPosition,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PositionEntry {
    key: OwnershipKey,
    position: OwnershipPosition,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OwnershipWatermark {
    pub key: OwnershipKey,
    pub side: OrderSide,
    pub ordered_quantity: u64,
    pub cumulative_quantity: u64,
    pub cumulative_notional: u128,
    last_application: AppliedRecord,
}

/// Source binding and the last positive fill's exact result. This position is a
/// historical snapshot; later orders and protection metadata may change runtime.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppliedRecord {
    key: OwnershipKey,
    side: OrderSide,
    ordered_quantity: u64,
    cumulative_quantity: u64,
    cumulative_notional: u128,
    applied: OwnershipApplied,
}

/// Retains open positions and only the latest cumulative watermark per order.
/// The caller persists a mutated clone before publishing it to runtime consumers.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OwnershipLedger {
    version: u32,
    #[serde(serialize_with = "serialize_generation")]
    generation: uuid::Uuid,
    positions: Vec<PositionEntry>,
    watermarks: BTreeMap<String, OwnershipWatermark>,
    metadata: Vec<PositionMetadata>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PositionMetadata {
    key: OwnershipKey,
    value: serde_json::Value,
}

impl Default for OwnershipLedger {
    fn default() -> Self {
        Self {
            version: VERSION,
            generation: uuid::Uuid::new_v4(),
            positions: Vec::new(),
            watermarks: BTreeMap::new(),
            metadata: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredLedger {
    version: u32,
    #[serde(deserialize_with = "deserialize_generation")]
    generation: uuid::Uuid,
    #[serde(deserialize_with = "bounded_positions")]
    positions: Vec<PositionEntry>,
    #[serde(deserialize_with = "unique_watermarks")]
    watermarks: BTreeMap<String, OwnershipWatermark>,
    #[serde(default, deserialize_with = "bounded_metadata")]
    metadata: Vec<PositionMetadata>,
}

fn serialize_generation<S>(
    generation: &uuid::Uuid,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&generation.to_string())
}

fn deserialize_generation<'de, D>(deserializer: D) -> std::result::Result<uuid::Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let generation = String::deserialize(deserializer)?;
    uuid::Uuid::parse_str(&generation).map_err(de::Error::custom)
}

fn bounded_metadata<'de, D>(deserializer: D) -> std::result::Result<Vec<PositionMetadata>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Bounded;
    impl<'de> Visitor<'de> for Bounded {
        type Value = Vec<PositionMetadata>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded list of position protection metadata")
        }
        fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = Vec::new();
            while let Some(entry) = sequence.next_element()? {
                if result.len() >= MAX_POSITIONS {
                    return Err(de::Error::custom("ownership metadata capacity exceeded"));
                }
                result.push(entry);
            }
            Ok(result)
        }
    }
    deserializer.deserialize_seq(Bounded)
}

fn validate_metadata(value: &serde_json::Value) -> Result<()> {
    let mut pending = vec![(value, 0_usize)];
    let mut nodes = 0_usize;
    while let Some((node, depth)) = pending.pop() {
        nodes += 1;
        ensure!(
            depth <= MAX_METADATA_DEPTH && nodes <= MAX_METADATA_BYTES,
            "ownership metadata is too deeply nested or large"
        );
        let children: Vec<_> = match node {
            serde_json::Value::Array(values) => {
                ensure!(
                    values.len() <= MAX_METADATA_BYTES,
                    "ownership metadata is too large"
                );
                values.iter().collect()
            }
            serde_json::Value::Object(values) => {
                ensure!(
                    values.len() <= MAX_METADATA_BYTES,
                    "ownership metadata is too large"
                );
                values.values().collect()
            }
            _ => Vec::new(),
        };
        ensure!(
            pending.len() + children.len() <= MAX_METADATA_BYTES,
            "ownership metadata is too large"
        );
        pending.extend(children.into_iter().map(|child| (child, depth + 1)));
    }
    struct SizeLimit(usize);
    impl std::io::Write for SizeLimit {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let next = self
                .0
                .checked_add(bytes.len())
                .filter(|size| *size <= MAX_METADATA_BYTES)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "ownership metadata exceeds 64 KiB",
                    )
                })?;
            self.0 = next;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(SizeLimit(0), value)?;
    Ok(())
}

fn bounded_positions<'de, D>(deserializer: D) -> std::result::Result<Vec<PositionEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Bounded;
    impl<'de> Visitor<'de> for Bounded {
        type Value = Vec<PositionEntry>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded list of ownership positions")
        }
        fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = Vec::new();
            while let Some(entry) = sequence.next_element()? {
                if result.len() >= MAX_POSITIONS {
                    return Err(de::Error::custom("ownership position capacity exceeded"));
                }
                result.push(entry);
            }
            Ok(result)
        }
    }
    deserializer.deserialize_seq(Bounded)
}

fn unique_watermarks<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, OwnershipWatermark>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Unique;
    impl<'de> Visitor<'de> for Unique {
        type Value = BTreeMap<String, OwnershipWatermark>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded map with unique order record IDs")
        }
        fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut result = BTreeMap::new();
            while let Some((id, watermark)) = map.next_entry::<String, OwnershipWatermark>()? {
                if result.len() >= MAX_WATERMARKS {
                    return Err(de::Error::custom("ownership watermark capacity exceeded"));
                }
                if result.insert(id, watermark).is_some() {
                    return Err(de::Error::custom("duplicate ownership order record ID"));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Unique)
}

impl<'de> Deserialize<'de> for OwnershipLedger {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stored = StoredLedger::deserialize(deserializer)?;
        let ledger = Self {
            version: stored.version,
            generation: stored.generation,
            positions: stored.positions,
            watermarks: stored.watermarks,
            metadata: stored.metadata,
        };
        ledger.validate().map_err(de::Error::custom)?;
        Ok(ledger)
    }
}

fn explicit_optional_profile<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

fn valid_id(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_ID_BYTES && !value.chars().any(char::is_control)
}

impl OwnershipKey {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.identity
                .scope
                .account_id
                .as_ref()
                .is_some_and(|account| valid_id(&account.0)),
            "ownership account identity is missing or invalid"
        );
        ensure!(
            self.identity.profile_id.as_deref().is_none_or(valid_id),
            "ownership profile identity is invalid"
        );
        if let OwnershipOwner::Strategy(id) = &self.owner {
            ensure!(valid_id(id), "ownership strategy identity is invalid");
        }
        ensure!(valid_id(&self.symbol), "ownership symbol is invalid");
        Ok(())
    }
}

fn cumulative(quantity: Option<u64>, notional: Option<u128>, ordered: u64) -> Result<(u64, u128)> {
    let quantity = quantity.context("confirmed ownership quantity is unknown")?;
    let notional = notional.context("confirmed ownership consideration is unknown")?;
    ensure!(
        ordered > 0 && quantity <= ordered,
        "ownership cumulative quantity exceeds the order"
    );
    ensure!(
        (quantity == 0 && notional == 0) || (quantity > 0 && notional > 0),
        "confirmed ownership quantity and consideration are inconsistent"
    );
    Ok((quantity, notional))
}

fn signed_difference(proceeds: u128, cost: u128) -> Result<i128> {
    if proceeds >= cost {
        return i128::try_from(proceeds - cost).context("ownership profit overflow");
    }
    let loss = cost - proceeds;
    if loss == (i128::MAX as u128) + 1 {
        return Ok(i128::MIN);
    }
    i128::try_from(loss)
        .map(|value| -value)
        .context("ownership loss overflow")
}

fn proportional_cost(cost: u128, quantity: u64, sold: u64) -> Result<u128> {
    ensure!(
        quantity > 0 && sold <= quantity,
        "invalid ownership released quantity"
    );
    let held = u128::from(quantity);
    let sold = u128::from(sold);
    (cost / held)
        .checked_mul(sold)
        .and_then(|whole| {
            (cost % held)
                .checked_mul(sold)
                .and_then(|remainder| whole.checked_add(remainder / held))
        })
        .context("ownership released cost overflow")
}

fn validate_application(watermark: &OwnershipWatermark) -> Result<()> {
    let record = &watermark.last_application;
    ensure!(
        record.key == watermark.key
            && record.side == watermark.side
            && record.ordered_quantity == watermark.ordered_quantity
            && record.cumulative_quantity == watermark.cumulative_quantity
            && record.cumulative_notional == watermark.cumulative_notional,
        "ownership application source binding changed"
    );
    let applied = &record.applied;
    ensure!(
        applied.delta_quantity > 0
            && applied.delta_quantity <= watermark.cumulative_quantity
            && applied.delta_notional > 0
            && applied.delta_notional <= watermark.cumulative_notional,
        "invalid ownership application cumulative delta"
    );
    ensure!(
        (applied.delta_quantity == watermark.cumulative_quantity)
            == (applied.delta_notional == watermark.cumulative_notional),
        "ownership application prior cumulative amounts are inconsistent"
    );
    ensure!(
        (applied.position.quantity == 0 && applied.position.cost_notional == 0)
            || (applied.position.quantity > 0 && applied.position.cost_notional > 0),
        "invalid ownership application position snapshot"
    );
    match watermark.side {
        OrderSide::Buy => {
            ensure!(
                applied.released_cost == 0
                    && applied.realized_pnl == 0
                    && applied.position.quantity >= applied.delta_quantity
                    && applied.position.cost_notional >= applied.delta_notional,
                "invalid ownership buy application result"
            );
            ensure!(
                (applied.position.quantity == applied.delta_quantity)
                    == (applied.position.cost_notional == applied.delta_notional),
                "ownership buy application prior position is inconsistent"
            );
        }
        OrderSide::Sell => {
            let before_quantity = applied
                .position
                .quantity
                .checked_add(applied.delta_quantity)
                .context("ownership application quantity overflow")?;
            let before_cost = applied
                .position
                .cost_notional
                .checked_add(applied.released_cost)
                .context("ownership application cost overflow")?;
            ensure!(
                before_cost > 0
                    && proportional_cost(before_cost, before_quantity, applied.delta_quantity)?
                        == applied.released_cost,
                "invalid ownership proportional sell application cost"
            );
            ensure!(
                signed_difference(applied.delta_notional, applied.released_cost)?
                    == applied.realized_pnl,
                "invalid ownership sell application profit"
            );
        }
    }
    Ok(())
}

impl OwnershipLedger {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.version == VERSION,
            "unsupported ownership ledger version"
        );
        ensure!(
            self.positions.len() <= MAX_POSITIONS
                && self.watermarks.len() <= MAX_WATERMARKS
                && self.metadata.len() <= MAX_POSITIONS,
            "ownership ledger capacity exceeded"
        );
        ensure!(
            !self.generation.is_nil(),
            "ownership ledger generation must not be nil"
        );
        let mut seen = HashSet::new();
        for entry in &self.positions {
            entry.key.validate()?;
            ensure!(
                entry.position.quantity > 0 && entry.position.cost_notional > 0,
                "invalid ownership position cost or quantity"
            );
            // Keys contain broker enums without Ord; the stable serialized value
            // is used only to detect duplicates during an explicit restore.
            ensure!(
                seen.insert(serde_json::to_string(&entry.key)?),
                "duplicate ownership position key"
            );
        }
        let mut metadata_keys = HashSet::new();
        for metadata in &self.metadata {
            metadata.key.validate()?;
            ensure!(
                self.position(&metadata.key).quantity > 0,
                "ownership metadata has no confirmed held position"
            );
            ensure!(
                metadata_keys.insert(serde_json::to_string(&metadata.key)?),
                "duplicate ownership metadata key"
            );
            validate_metadata(&metadata.value)?;
        }
        for (id, watermark) in &self.watermarks {
            ensure!(valid_id(id), "ownership order record ID is invalid");
            watermark.key.validate()?;
            cumulative(
                Some(watermark.cumulative_quantity),
                Some(watermark.cumulative_notional),
                watermark.ordered_quantity,
            )?;
            validate_application(watermark)?;
        }
        Ok(())
    }

    pub(crate) fn generation(&self) -> uuid::Uuid {
        self.generation
    }

    pub(crate) fn position_metadata(&self, key: &OwnershipKey) -> Option<&serde_json::Value> {
        self.metadata
            .iter()
            .find(|metadata| &metadata.key == key)
            .map(|metadata| &metadata.value)
    }

    /// Accept only strategy protection exports; credentials must never enter this
    /// opaque payload. Broker-wide holdings do not authorize metadata adoption.
    pub(crate) fn set_position_metadata(
        &mut self,
        key: &OwnershipKey,
        value: serde_json::Value,
    ) -> Result<()> {
        key.validate()?;
        ensure!(
            self.position(key).quantity > 0,
            "ownership metadata requires confirmed held quantity"
        );
        validate_metadata(&value)?;
        let index = self
            .metadata
            .iter()
            .position(|metadata| &metadata.key == key);
        ensure!(
            index.is_some() || self.metadata.len() < MAX_POSITIONS,
            "ownership metadata capacity exceeded"
        );
        if let Some(index) = index {
            self.metadata[index].value = value;
        } else {
            self.metadata.push(PositionMetadata {
                key: key.clone(),
                value,
            });
        }
        Ok(())
    }

    pub(crate) fn position(&self, key: &OwnershipKey) -> OwnershipPosition {
        self.positions
            .iter()
            .find(|entry| &entry.key == key)
            .map_or_else(OwnershipPosition::default, |entry| entry.position)
    }

    pub(crate) fn positions(&self) -> impl Iterator<Item = (&OwnershipKey, OwnershipPosition)> {
        self.positions
            .iter()
            .map(|entry| (&entry.key, entry.position))
    }

    pub(crate) fn order_watermark(&self, order_record_id: &str) -> Option<&OwnershipWatermark> {
        self.watermarks.get(order_record_id)
    }

    /// Exact result from this order's latest positive cumulative application.
    /// Replays still return delta zero; callers must first match the watermark's
    /// exact cumulative source before using this summary to recover durable effects.
    pub(crate) fn last_application(&self, order_record_id: &str) -> Option<OwnershipApplied> {
        self.watermarks
            .get(order_record_id)
            .map(|watermark| watermark.last_application.applied.clone())
    }

    /// All validation and checked arithmetic finish before the first mutation.
    /// A rejected application leaves positions and watermarks byte-for-byte intact.
    pub(crate) fn apply_cumulative(&mut self, fill: &OwnershipFill) -> Result<OwnershipApplied> {
        fill.key.validate()?;
        ensure!(
            valid_id(&fill.order_record_id),
            "ownership order record ID is invalid"
        );
        let (quantity, notional) = cumulative(
            fill.cumulative_quantity,
            fill.cumulative_notional,
            fill.ordered_quantity,
        )?;
        let previous = self.watermarks.get(&fill.order_record_id);
        if let Some(previous) = previous {
            ensure!(
                previous.key == fill.key
                    && previous.side == fill.side
                    && previous.ordered_quantity == fill.ordered_quantity,
                "ownership order identity changed"
            );
        }
        let (previous_quantity, previous_notional) = previous.map_or((0, 0), |previous| {
            (previous.cumulative_quantity, previous.cumulative_notional)
        });
        let position = self.position(&fill.key);
        if quantity <= previous_quantity && notional <= previous_notional {
            return Ok(OwnershipApplied {
                delta_quantity: 0,
                delta_notional: 0,
                released_cost: 0,
                realized_pnl: 0,
                position,
            });
        }
        ensure!(
            quantity > previous_quantity && notional > previous_notional,
            "ownership cumulative amounts changed without a positive fill delta"
        );
        let delta_quantity = quantity - previous_quantity;
        let delta_notional = notional - previous_notional;
        let (next_position, released_cost, realized_pnl) = match fill.side {
            OrderSide::Buy => {
                let held = position
                    .quantity
                    .checked_add(delta_quantity)
                    .context("ownership quantity overflow")?;
                let cost = position
                    .cost_notional
                    .checked_add(delta_notional)
                    .context("ownership cost overflow")?;
                (
                    OwnershipPosition {
                        quantity: held,
                        cost_notional: cost,
                    },
                    0,
                    0,
                )
            }
            OrderSide::Sell => {
                ensure!(
                    position.quantity >= delta_quantity && position.cost_notional > 0,
                    "confirmed sell exceeds this owner's known holdings or cost basis"
                );
                // Quotient/remainder arithmetic preserves floor(cost * sold / held)
                // without overflowing an intermediate product whose result fits.
                let released =
                    proportional_cost(position.cost_notional, position.quantity, delta_quantity)?;
                let next = OwnershipPosition {
                    quantity: position.quantity - delta_quantity,
                    cost_notional: position.cost_notional - released,
                };
                ensure!(
                    next.quantity > 0 || next.cost_notional == 0,
                    "ownership final sale left a cost basis"
                );
                let pnl = signed_difference(delta_notional, released)?;
                (next, released, pnl)
            }
        };
        let index = self
            .positions
            .iter()
            .position(|entry| entry.key == fill.key);
        ensure!(
            index.is_some() || next_position.quantity == 0 || self.positions.len() < MAX_POSITIONS,
            "ownership position capacity exceeded"
        );
        ensure!(
            previous.is_some() || self.watermarks.len() < MAX_WATERMARKS,
            "ownership watermark capacity exceeded"
        );
        if let Some(index) = index {
            if next_position.quantity == 0 {
                self.positions.remove(index);
                self.metadata.retain(|metadata| metadata.key != fill.key);
            } else {
                self.positions[index].position = next_position;
            }
        } else if next_position.quantity > 0 {
            self.positions.push(PositionEntry {
                key: fill.key.clone(),
                position: next_position,
            });
        }
        let applied = OwnershipApplied {
            delta_quantity,
            delta_notional,
            released_cost,
            realized_pnl,
            position: next_position,
        };
        self.watermarks.insert(
            fill.order_record_id.clone(),
            OwnershipWatermark {
                key: fill.key.clone(),
                side: fill.side,
                ordered_quantity: fill.ordered_quantity,
                cumulative_quantity: quantity,
                cumulative_notional: notional,
                last_application: AppliedRecord {
                    key: fill.key.clone(),
                    side: fill.side,
                    ordered_quantity: fill.ordered_quantity,
                    cumulative_quantity: quantity,
                    cumulative_notional: notional,
                    applied: applied.clone(),
                },
            },
        );
        Ok(applied)
    }

    /// Only call after the exact terminal cumulative fill and pending removal are
    /// durably confirmed. Pruning relinquishes replay protection for that ID; an
    /// archived terminal event must never be fed back after this explicit step.
    pub(crate) fn prune_terminal_order(&mut self, terminal: &OwnershipFill) -> Result<bool> {
        terminal.key.validate()?;
        ensure!(
            valid_id(&terminal.order_record_id),
            "ownership order record ID is invalid"
        );
        let (quantity, notional) = cumulative(
            terminal.cumulative_quantity,
            terminal.cumulative_notional,
            terminal.ordered_quantity,
        )?;
        let Some(previous) = self.watermarks.get(&terminal.order_record_id) else {
            return Ok(false);
        };
        ensure!(
            previous.key == terminal.key
                && previous.side == terminal.side
                && previous.ordered_quantity == terminal.ordered_quantity
                && previous.cumulative_quantity == quantity
                && previous.cumulative_notional == notional,
            "ownership terminal proof differs from the applied watermark"
        );
        self.watermarks.remove(&terminal.order_record_id);
        Ok(true)
    }
}

#[cfg(test)]
mod tests;

mod runtime;
