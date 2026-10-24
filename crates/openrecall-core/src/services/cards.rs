use super::library::id;
use crate::{
    App, AppError, AppResult,
    contracts::{
        cards::{CardDraft, Page},
        library::{DeckDetail, DeleteReport},
    },
    domain::card::{CardDetail, CardForm},
};
use chrono::DateTime;
impl App {
    pub(crate) fn form_drafts(
        &self,
        draft: CardDraft,
        old: Option<&CardDetail>,
    ) -> AppResult<Vec<CardForm>> {
        if draft.forms.is_empty() {
            return Err(AppError::field("forms", "required"));
        }
        let mut used = std::collections::HashSet::new();
        draft
            .forms
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                if f.front.trim().is_empty() {
                    return Err(AppError::field(&format!("forms.{i}.front"), "required"));
                }
                if f.back.trim().is_empty() {
                    return Err(AppError::field(&format!("forms.{i}.back"), "required"));
                }
                let form_id = match f.id {
                    Some(existing) => {
                        if old.is_none_or(|c| !c.forms.iter().any(|item| item.id == existing)) {
                            return Err(AppError::field("forms", "invalidOwner"));
                        }
                        existing
                    }
                    None => id(),
                };
                if !used.insert(form_id.clone()) {
                    return Err(AppError::field("forms", "duplicateId"));
                }
                Ok(CardForm {
                    id: form_id,
                    front: f.front,
                    back: f.back,
                    notes: f.notes,
                    language: f.language.filter(|s| !s.trim().is_empty()),
                })
            })
            .collect()
    }
    pub(crate) fn deck_detail(&self, deck_id: &str) -> AppResult<DeckDetail> {
        let mut deck: DeckDetail = self.store.get("decks", deck_id)?;
        let counts=self.store.conn.query_row("SELECT count(*),coalesce(sum(due_ms<=?2),0),coalesce(sum(review_count=0),0) FROM cards WHERE deck_id=?1",rusqlite::params![deck_id,self.clock.now().timestamp_millis()],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,u32>(1)?,r.get::<_,u32>(2)?)))?;
        (deck.logical_cards, deck.due_now, deck.new_cards) = counts;
        Ok(deck)
    }
    pub(crate) fn deck_list(&self) -> AppResult<Vec<DeckDetail>> {
        self.store
            .all::<DeckDetail>("decks")?
            .into_iter()
            .map(|d| self.deck_detail(&d.id))
            .collect()
    }
    pub(crate) fn touch_deck(&self, deck_id: &str) -> AppResult<()> {
        let mut deck: DeckDetail = self.store.get("decks", deck_id)?;
        deck.revision = id();
        deck.data_revision = id();
        self.save_deck(&deck)?;
        self.store.touch()
    }
    pub(crate) fn save_card(&self, card: &CardDetail) -> AppResult<()> {
        let due = DateTime::parse_from_rfc3339(&card.due_at)
            .map_err(|_| AppError::new("invalidData"))?
            .timestamp_millis();
        let created = DateTime::parse_from_rfc3339(&card.created_at)
            .map_err(|_| AppError::new("invalidData"))?
            .timestamp_millis();
        self.store.conn.execute("INSERT INTO cards(id,deck_id,due_ms,created_ms,review_count,payload) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET due_ms=excluded.due_ms,review_count=excluded.review_count,payload=excluded.payload",rusqlite::params![card.id,card.deck_id,due,created,card.review_count,serde_json::to_string(card)?])?;
        Ok(())
    }
    pub(crate) fn create_card(
        &mut self,
        deck_id: String,
        draft: CardDraft,
    ) -> AppResult<CardDetail> {
        self.create_card_at(deck_id, draft, self.clock.now())
    }
    pub(crate) fn create_card_at(
        &mut self,
        deck_id: String,
        draft: CardDraft,
        at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<CardDetail> {
        let _: DeckDetail = self.store.get("decks", &deck_id)?;
        let forms = self.form_drafts(draft, None)?;
        let now = at.to_rfc3339();
        let card = CardDetail {
            bag: Default::default(),
            id: id(),
            deck_id: deck_id.clone(),
            forms,
            phase: "new".into(),
            memory: None,
            due_at: now.clone(),
            created_at: now,
            reviewed_at: None,
            review_count: 0,
            lapse_count: 0,
            has_graduated: false,
            revision: id(),
        };
        self.save_card(&card)?;
        self.touch_deck(&deck_id)?;
        Ok(card)
    }
    pub(crate) fn update_card(
        &mut self,
        card_id: String,
        draft: CardDraft,
        expected: String,
    ) -> AppResult<CardDetail> {
        let mut card: CardDetail = self.store.get("cards", &card_id)?;
        if card.revision != expected {
            return Err(AppError::new("conflict"));
        }
        card.forms = self.form_drafts(draft, Some(&card))?;
        self.invalidate_card_undo(&card.id)?;
        let ids = card.forms.iter().map(|f| f.id.clone()).collect::<Vec<_>>();
        crate::scheduling::forms::reconcile_forms(&mut card.bag, &ids, self.random.as_mut())?;
        card.revision = id();
        self.save_card(&card)?;
        self.touch_deck(&card.deck_id)?;
        Ok(card)
    }
    pub(crate) fn delete_card(
        &mut self,
        card_id: String,
        expected: String,
    ) -> AppResult<DeleteReport> {
        let card: CardDetail = self.store.get("cards", &card_id)?;
        if card.revision != expected {
            return Err(AppError::new("conflict"));
        }
        self.invalidate_card_undo(&card_id)?;
        self.store
            .conn
            .execute("DELETE FROM cards WHERE id=?1", [&card_id])?;
        self.refresh_session_counts(&card.deck_id)?;
        self.touch_deck(&card.deck_id)?;
        Ok(DeleteReport { id: card_id })
    }
    pub(crate) fn list_cards(
        &self,
        deck_id: String,
        query: String,
        page: u32,
        page_size: u16,
    ) -> AppResult<Page<CardDetail>> {
        let _: DeckDetail = self.store.get("decks", &deck_id)?;
        if page == 0 || ![10, 20, 50, 100].contains(&page_size) {
            return Err(AppError::field("page", "range"));
        }
        let filter = "deck_id=?1 AND (?2='' OR EXISTS(SELECT 1 FROM json_each(cards.payload,'$.forms') f WHERE instr(lower(json_extract(f.value,'$.front')||' '||json_extract(f.value,'$.back')||' '||json_extract(f.value,'$.notes')),lower(?2))>0))";
        let total = self.store.conn.query_row(
            &format!("SELECT count(*) FROM cards WHERE {filter}"),
            rusqlite::params![deck_id, query],
            |r| r.get(0),
        )?;
        let mut stmt = self.store.conn.prepare(&format!(
            "SELECT payload FROM cards WHERE {filter} ORDER BY created_ms,id LIMIT ?3 OFFSET ?4"
        ))?;
        let items = stmt
            .query_map(
                rusqlite::params![
                    deck_id,
                    query,
                    page_size,
                    (page as i64 - 1) * page_size as i64
                ],
                |r| r.get::<_, String>(0),
            )?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Page {
            items,
            total,
            page,
            page_size,
        })
    }
}
