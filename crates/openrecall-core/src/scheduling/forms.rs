use crate::{AppError, AppResult, RandomSource};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct FormBag {
    pub remaining: Vec<String>,
    pub consumed: Vec<String>,
    pub last_consumed: Option<String>,
    pub reserved: Option<String>,
    pub cycle: u32,
}
pub fn reserve_form(
    bag: &mut FormBag,
    ids: &[String],
    rng: &mut dyn RandomSource,
) -> AppResult<String> {
    reconcile_forms(bag, ids, rng)?;
    if bag.remaining.is_empty() {
        bag.remaining = ids.to_vec();
        rng.shuffle(&mut bag.remaining);
        bag.consumed.clear();
        bag.cycle = bag.cycle.saturating_add(1);
        if bag.remaining.len() > 1 && bag.last_consumed.as_ref() == bag.remaining.first() {
            bag.remaining.swap(0, 1);
        }
    }
    if bag.cycle == 0 {
        bag.cycle = 1;
    }
    let chosen = bag
        .reserved
        .clone()
        .unwrap_or_else(|| bag.remaining[0].clone());
    bag.reserved = Some(chosen.clone());
    Ok(chosen)
}
pub fn consume_form(bag: &mut FormBag, id: &str) -> AppResult<()> {
    if bag.reserved.as_deref() != Some(id) || bag.remaining.first().map(String::as_str) != Some(id)
    {
        return Err(AppError::new("invalidPresentation"));
    }
    bag.remaining.remove(0);
    bag.consumed.push(id.into());
    bag.last_consumed = Some(id.into());
    bag.reserved = None;
    Ok(())
}
pub fn reconcile_forms(
    bag: &mut FormBag,
    ids: &[String],
    rng: &mut dyn RandomSource,
) -> AppResult<()> {
    if ids.is_empty() {
        return Err(AppError::new("noForms"));
    }
    let valid: std::collections::HashSet<_> = ids.iter().collect();
    if valid.len() != ids.len() {
        return Err(AppError::new("duplicateFormId"));
    }
    bag.remaining.retain(|id| valid.contains(id));
    bag.consumed.retain(|id| valid.contains(id));
    if bag
        .reserved
        .as_ref()
        .is_some_and(|id| !bag.remaining.contains(id))
    {
        bag.reserved = None;
    }
    let added: Vec<_> = ids
        .iter()
        .filter(|id| !bag.remaining.contains(id) && !bag.consumed.contains(id))
        .cloned()
        .collect();
    if !added.is_empty() {
        bag.remaining.extend(added);
        if let Some(reserved) = bag.reserved.as_ref() {
            let at = bag
                .remaining
                .iter()
                .position(|id| id == reserved)
                .ok_or_else(|| AppError::new("invalidPresentation"))?;
            bag.remaining.swap(0, at);
            rng.shuffle(&mut bag.remaining[1..]);
        } else {
            rng.shuffle(&mut bag.remaining);
        }
    }
    Ok(())
}
